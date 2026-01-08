async function main(component: Input): Promise<Output> {
  // Get the generated code from code gen function
  const codeString = component.properties.code?.["Google Cloud Update Code Gen"]?.code;
  if (!codeString) {
    return {
      status: "error",
      message: "Could not find Google Cloud Update Code Gen code for resource",
    };
  }

  let updatePayload = JSON.parse(codeString);

  // Get current resource state to compare
  const currentResource = component.properties?.resource?.payload;

  // Filter to only changed fields (GCP PATCH requires this for some resources)
  // Include fields if they're different from current resource OR if they're being set for the first time
  if (currentResource) {
    const changedFields: Record<string, any> = {};

    for (const [key, value] of Object.entries(updatePayload)) {
      // Include field if:
      // 1. It doesn't exist in current resource (new field being set)
      // 2. OR it exists but has a different value
      if (!(key in currentResource) || !_.isEqual(value, currentResource[key])) {
        changedFields[key] = value;
      }
    }

    updatePayload = changedFields;

    // GCP requires fingerprint for updates to prevent concurrent modifications
    // Always include the fingerprint from the current resource if it exists
    if (currentResource.fingerprint) {
      updatePayload.fingerprint = currentResource.fingerprint;
    }
  }

  // Try to get update API path first, fall back to patch
  let updateApiPathJson = _.get(
    component.properties,
    ["domain", "extra", "updateApiPath"],
    "",
  );

  if (!updateApiPathJson) {
    updateApiPathJson = _.get(
      component.properties,
      ["domain", "extra", "patchApiPath"],
      "",
    );
  }

  if (!updateApiPathJson) {
    return {
      status: "error",
      message: "No update or patch API path metadata found - this resource may not support updates",
    };
  }

  const updateApiPath = JSON.parse(updateApiPathJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");

  // Get resourceId
  const resourceId = component.properties?.si?.resourceId;
  if (!resourceId) {
    return {
      status: "error",
      message: "No resource ID found for update",
    };
  }

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  // Build the URL by replacing path parameters
  let url = `${baseUrl}${updateApiPath.path}`;

  // Replace path parameters with values from resource_value or domain
  // GCP APIs use RFC 6570 URI templates: {param} and {+param} (reserved expansion)
  if (updateApiPath.parameterOrder) {
    for (const paramName of updateApiPath.parameterOrder) {
      let paramValue;

      // For the resource identifier, use resourceId
      if (paramName === updateApiPath.parameterOrder[updateApiPath.parameterOrder.length - 1]) {
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

  // Many GCP APIs require or benefit from an updateMask query parameter
  // that specifies which fields are being updated
  const updateFields = Object.keys(updatePayload).filter(k => k !== 'fingerprint');
  if (updateFields.length > 0) {
    const updateMask = updateFields.join(',');
    url += (url.includes('?') ? '&' : '?') + `updateMask=${encodeURIComponent(updateMask)}`;
  }

  // Make the API request with retry logic
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: updateApiPath.httpMethod,
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(updatePayload),
    });

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`Unable to update resource; API returned ${resp.status} ${resp.statusText}: ${errorText}`) as any;
      error.status = resp.status;
      error.body = errorText;
      throw error;
    }

    return resp;
  }, {
    isRateLimitedFn: (error) => error.status === 429
  }).then((r) => r.result);

  const responseJson = await response.json();

  // Handle Google Cloud Long-Running Operations (LRO)
  // Check if this is an operation response:
  // - Compute Engine uses "kind" containing "operation"
  // - GKE/Container API uses "operationType" field
  // - Other APIs (API Keys, etc.) use "name" starting with "operations/"
  const isLRO = (responseJson.kind && responseJson.kind.includes("operation")) ||
                responseJson.operationType ||
                (responseJson.name && responseJson.name.startsWith("operations/"));
  if (isLRO) {
    console.log(`[UPDATE] LRO detected, polling for completion...`);

    // Use selfLink or construct URL from operation name
    // For APIs that don't provide selfLink, we need to construct the URL
    // The API version prefix (v1, v2, etc.) comes from the API paths
    let pollingUrl = responseJson.selfLink;
    if (!pollingUrl && responseJson.name) {
      // Extract version from one of the API paths (e.g., "v2/{+parent}/keys" -> "v2")
      const insertApiPathJson = _.get(component.properties, ["domain", "extra", "insertApiPath"], "");
      const getApiPathJson = _.get(component.properties, ["domain", "extra", "getApiPath"], "");
      const pathJson = insertApiPathJson || getApiPathJson;
      let apiVersion = "";
      if (pathJson) {
        const apiPath = JSON.parse(pathJson);
        const versionMatch = apiPath.path?.match(/^(v\d+)\//);
        if (versionMatch) {
          apiVersion = versionMatch[1] + "/";
        }
      }
      pollingUrl = `${baseUrl}${apiVersion}${responseJson.name}`;
    }

    // Poll the operation until it completes using new siExec.pollLRO
    const finalResource = await siExec.pollLRO({
      url: pollingUrl,
      headers: { "Authorization": `Bearer ${token}` },
      maxAttempts: 20,
      baseDelay: 2000,
      maxDelay: 30000,
      isCompleteFn: (response: any, body: any) => body.status === "DONE" || body.done === true,
      isErrorFn: (response: any, body: any) => !!body.error,
      extractResultFn: async (response: any, body: any) => {
        // If operation has error, throw it
        if (body.error) {
          throw new Error(`Operation failed: ${JSON.stringify(body.error)}`);
        }

        // Some operations include the updated resource in the response field
        if (body.response) {
          return body.response;
        }

        // For update operations, fetch the final resource from targetLink
        if (body.targetLink) {
          const resourceResponse = await fetch(body.targetLink, {
            method: "GET",
            headers: { "Authorization": `Bearer ${token}` },
          });

          if (!resourceResponse.ok) {
            throw new Error(`Failed to fetch final resource: ${resourceResponse.status}`);
          }

          return await resourceResponse.json();
        }

        // Fallback: Use resourceId with getApiPath to fetch final resource
        // This handles APIs like API Keys that don't provide targetLink or response
        const getApiPathJson = _.get(component.properties, ["domain", "extra", "getApiPath"], "");
        if (getApiPathJson && resourceId) {
          const getApiPath = JSON.parse(getApiPathJson);
          let getUrl = `${baseUrl}${getApiPath.path}`;

          // Replace {+name} or {name} with resourceId
          if (getUrl.includes("{+name}")) {
            getUrl = getUrl.replace("{+name}", resourceId);
          } else if (getUrl.includes("{name}")) {
            getUrl = getUrl.replace("{name}", encodeURIComponent(resourceId));
          }

          const resourceResponse = await fetch(getUrl, {
            method: "GET",
            headers: { "Authorization": `Bearer ${token}` },
          });

          if (resourceResponse.ok) {
            return await resourceResponse.json();
          }
        }

        console.warn("[GCP] Operation completed but couldn't fetch final resource");
        return body;
      }
    });

    console.log(`[UPDATE] Operation complete`);
    return {
      payload: normalizeGcpResourceValues(finalResource),
      status: "ok",
    };
  }

  // Handle synchronous response
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
