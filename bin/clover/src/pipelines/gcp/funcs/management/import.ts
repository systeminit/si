async function main({ thisComponent }: Input): Promise<Output> {
  const component = thisComponent.properties;
  const resourcePayload = _.get(component, ["resource", "payload"], "");
  const resourceId = _.get(component, ["si", "resourceId"]);

  if (!resourceId) {
    return {
      status: "error",
      message: "No resourceId set, cannot import resource",
    };
  }

  // Get API path metadata from domain.extra
  const getApiPathJson = _.get(component, ["domain", "extra", "getApiPath"], "");
  if (!getApiPathJson) {
    return {
      status: "error",
      message: "No get API path metadata found - this resource may not support import",
    };
  }

  const getApiPath = JSON.parse(getApiPathJson);
  const baseUrl = _.get(component, ["domain", "extra", "baseUrl"], "");

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  // Build the URL
  const url = buildUrlWithParams(baseUrl, getApiPath, component, projectId, resourceId);

  // Make the API request with retry logic
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "GET",
      headers: { "Authorization": `Bearer ${token}` },
    });

    if (!resp.ok) {
      const errorText = await resp.text();
      console.log("Failed to import Google Cloud resource");
      console.error(errorText);
      const error = new Error(`Import error; API returned ${resp.status} ${resp.statusText}: ${errorText}`) as any;
      error.status = resp.status;
      error.body = errorText;
      throw error;
    }

    return resp;
  }, {
    isRateLimitedFn: (error: any) => error.status === 429
  }).then((r: any) => r.result);

  const resourceProperties = await response.json();
  console.log(resourceProperties);

  // Normalize GCP URLs to resource names
  const normalizedProperties = normalizeGcpResourceValues(resourceProperties);

  const properties = {
    ...component,
    domain: {
      ...component.domain,
      ...normalizedProperties,
    },
  };

  let needsRefresh = true;
  if (!resourcePayload) {
    properties.resource = normalizedProperties;
    needsRefresh = false;
  }

  const ops = {
    update: {
      self: { properties },
    },
    actions: {
      self: {
        remove: ["create"],
        add: [] as string[],
      },
    },
  };

  if (needsRefresh) {
    ops.actions.self.add.push("refresh");
  } else {
    ops.actions.self.remove.push("refresh");
  }

  return {
    status: "ok",
    message: "Imported Resource",
    ops,
  };
}

// ============================================================================
// Helper Functions
// ============================================================================

// Get location from component
function getLocation(component: any): string | undefined {
  return _.get(component, ["domain", "location"]) ||
    _.get(component, ["domain", "zone"]) ||
    _.get(component, ["domain", "region"]);
}

// Resolve a parameter value from component properties
function resolveParamValue(
  component: any,
  paramName: string,
  projectId: string | undefined
): string | undefined {
  if (paramName === "project" || paramName === "projectId") {
    return projectId;
  }

  if (paramName === "parent") {
    let parentValue = _.get(component, ["domain", "parent"]);
    if (!parentValue && projectId) {
      const location = getLocation(component);
      const supportsAutoConstruct = _.get(component, ["domain", "extra", "supportsParentAutoConstruct"]) === "true";

      if (supportsAutoConstruct && location) {
        parentValue = `projects/${projectId}/locations/${location}`;
      }
    }
    return parentValue;
  }

  return _.get(component, ["domain", paramName]);
}

// Build URL by replacing path parameters
function buildUrlWithParams(
  baseUrl: string,
  apiPath: { path: string; parameterOrder?: string[] },
  component: any,
  projectId: string | undefined,
  resourceId: string
): string {
  let url = `${baseUrl}${apiPath.path}`;

  if (apiPath.parameterOrder) {
    const lastParam = apiPath.parameterOrder[apiPath.parameterOrder.length - 1];

    for (const paramName of apiPath.parameterOrder) {
      let paramValue: string | undefined;

      if (paramName === lastParam) {
        paramValue = resourceId;
      } else {
        paramValue = resolveParamValue(component, paramName, projectId);
      }

      if (paramValue) {
        if (url.includes(`{+${paramName}}`)) {
          url = url.replace(`{+${paramName}}`, paramValue);
        } else if (url.includes(`{${paramName}}`)) {
          url = url.replace(`{${paramName}}`, encodeURIComponent(paramValue));
        }
      }
    }
  }

  return url;
}

async function getAccessToken(serviceAccountJson: string): Promise<{ token: string; projectId: string | undefined }> {
  let projectId: string | undefined;
  try {
    const serviceAccount = JSON.parse(serviceAccountJson);
    projectId = serviceAccount.project_id;
  } catch {
    projectId = undefined;
  }

  const activateResult = await siExec.waitUntilEnd("gcloud", [
    "auth", "activate-service-account", "--key-file=-", "--quiet"
  ], { input: serviceAccountJson });

  if (activateResult.exitCode !== 0) {
    throw new Error(`Failed to activate service account: ${activateResult.stderr}`);
  }

  const tokenResult = await siExec.waitUntilEnd("gcloud", ["auth", "print-access-token"]);
  if (tokenResult.exitCode !== 0) {
    throw new Error(`Failed to get access token: ${tokenResult.stderr}`);
  }

  return { token: tokenResult.stdout.trim(), projectId };
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
