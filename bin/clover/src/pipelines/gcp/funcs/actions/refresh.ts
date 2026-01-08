async function main(component: Input): Promise<Output> {
  // Get API path metadata from domain.extra
  const getApiPathJson = _.get(
    component.properties,
    ["domain", "extra", "getApiPath"],
    "",
  );

  if (!getApiPathJson) {
    return {
      status: "error",
      message: "No get API path metadata found - this resource may not support refresh",
    };
  }

  const getApiPath = JSON.parse(getApiPathJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");

  // Get resourceId
  const resourceId = component.properties?.si?.resourceId;
  if (!resourceId) {
    return {
      status: "error",
      message: "No resource ID found for refresh",
    };
  }

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  // Build the URL by replacing path parameters
  let url = `${baseUrl}${getApiPath.path}`;

  // Replace path parameters with values from resource_value or domain
  // GCP APIs use RFC 6570 URI templates: {param} and {+param} (reserved expansion)
  if (getApiPath.parameterOrder) {
    for (const paramName of getApiPath.parameterOrder) {
      let paramValue;

      // For the resource identifier, use resourceId
      if (paramName === getApiPath.parameterOrder[getApiPath.parameterOrder.length - 1]) {
        paramValue = resourceId;
      } else if (paramName === "project") {
        // Use extracted project_id for project parameter
        paramValue = projectId;
      } else if (paramName === "parent") {
        // "parent" is a common GCP pattern: projects/{project}/locations/{location}
        paramValue = _.get(component.properties, ["resource", "payload", "parent"]) ||
                     _.get(component.properties, ["domain", "parent"]);
        if (!paramValue && projectId) {
          // Only auto-construct for project-only resources
          // Multi-scope resources require explicit parent
          const availableScopesJson = _.get(component.properties, ["domain", "extra", "availableScopes"]);
          const availableScopes = availableScopesJson ? JSON.parse(availableScopesJson) : [];
          const isProjectOnly = availableScopes.length === 1 && availableScopes[0] === "projects";

          if (isProjectOnly) {
            const location = _.get(component.properties, ["resource", "payload", "location"]) ||
                            _.get(component.properties, ["domain", "location"]) ||
                            _.get(component.properties, ["domain", "zone"]) ||
                            _.get(component.properties, ["domain", "region"]);
            if (location) {
              paramValue = `projects/${projectId}/locations/${location}`;
            }
          }
        }
      } else {
        paramValue = _.get(component.properties, ["resource", "payload", paramName]) ||
                     _.get(component.properties, ["domain", paramName]);

        // GCP often returns full URLs for reference fields (e.g., region, zone, network)
        // Extract just the resource name from the URL
        if (paramValue && typeof paramValue === "string" && paramValue.startsWith("https://")) {
          const urlParts = paramValue.split("/");
          paramValue = urlParts[urlParts.length - 1];
        }
      }

      if (paramValue) {
        // Handle {+param} (reserved expansion - don't encode, allows slashes)
        if (url.includes(`{+${paramName}}`)) {
          url = url.replace(`{+${paramName}}`, paramValue);
        } else {
          // Handle {param} (simple expansion - encode)
          url = url.replace(`{${paramName}}`, encodeURIComponent(paramValue));
        }
      }
    }
  }

  // Make the API request with retry logic
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "GET", // get is always GET
      headers: {
        "Authorization": `Bearer ${token}`,
      },
    });

    if (!resp.ok) {
      // Check if resource was deleted (404 is OK for refresh)
      if (resp.status === 404) {
        return resp;
      }

      const errorText = await resp.text();
      const error = new Error(`Unable to refresh resource; API returned ${resp.status} ${resp.statusText}: ${errorText}`) as any;
      error.status = resp.status;
      error.body = errorText;
      throw error;
    }

    return resp;
  }, {
    isRateLimitedFn: (error) => error.status === 429
  }).then((r) => r.result);

  // Handle 404 as success for refresh operations (resource was deleted)
  if (response.status === 404) {
    return {
      status: "ok",
      payload: null,
    };
  }

  const responseJson = await response.json();

  return {
    payload: normalizeGcpResourceValues(responseJson),
    status: "ok",
  };
}

async function getAccessToken(serviceAccountJson: string): Promise<{ token: string; projectId: string | undefined }> {
  // Parse service account JSON to extract project_id (optional)
  let projectId: string | undefined;
  try {
    const serviceAccount = JSON.parse(serviceAccountJson);
    projectId = serviceAccount.project_id;
  } catch {
    // If parsing fails or project_id is missing, continue without it
    projectId = undefined;
  }

  const activateResult = await siExec.waitUntilEnd("gcloud", [
    "auth",
    "activate-service-account",
    "--key-file=-",
    "--quiet"
  ], {
    input: serviceAccountJson
  });

  if (activateResult.exitCode !== 0) {
    throw new Error(`Failed to activate service account: ${activateResult.stderr}`);
  }

  const tokenResult = await siExec.waitUntilEnd("gcloud", [
    "auth",
    "print-access-token"
  ]);

  if (tokenResult.exitCode !== 0) {
    throw new Error(`Failed to get access token: ${tokenResult.stderr}`);
  }

  return {
    token: tokenResult.stdout.trim(),
    projectId,
  };
}

// URL normalization for GCP resource values
const GCP_URL_PATTERN = /^https:\/\/[^/]*\.?googleapis\.com\//;
const LOCATION_SEGMENTS = new Set(["regions", "zones", "locations"]);

function normalizeGcpResourceValues<T>(obj: T): T {
  if (obj === null || obj === undefined) return obj;
  if (Array.isArray(obj)) return obj.map(item => normalizeGcpResourceValues(item)) as T;
  if (typeof obj === "object") {
    const normalized: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj)) {
      if (typeof value === "string" && GCP_URL_PATTERN.test(value)) {
        const pathParts = new URL(value).pathname.split("/").filter(Boolean);
        if (pathParts.length >= 2 && LOCATION_SEGMENTS.has(pathParts[pathParts.length - 2])) {
          normalized[key] = pathParts[pathParts.length - 1];
        } else {
          const projectsIdx = pathParts.indexOf("projects");
          if (projectsIdx !== -1) {
            normalized[key] = pathParts.slice(projectsIdx).join("/");
          } else {
            // For non-project APIs (e.g., Storage), extract after API version (v1, v2, etc.)
            const versionIdx = pathParts.findIndex(p => /^v\d+/.test(p));
            normalized[key] = versionIdx !== -1 && versionIdx + 1 < pathParts.length
              ? pathParts.slice(versionIdx + 1).join("/")
              : pathParts[pathParts.length - 1] || value;
          }
        }
      } else if (typeof value === "object" && value !== null) {
        normalized[key] = normalizeGcpResourceValues(value);
      } else {
        normalized[key] = value;
      }
    }
    return normalized as T;
  }
  return obj;
}
