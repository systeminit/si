async function main(component: Input): Promise<Output> {
  // Check if resource already exists
  const existingPayload = component.properties.resource?.payload;
  if (existingPayload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: existingPayload,
    };
  }

  // Get the generated code from code gen function
  const codeString = component.properties.code?.["Google Cloud Create Code Gen"]?.code;
  if (!codeString) {
    return {
      status: "error",
      message: "Could not find Google Cloud Create Code Gen code for resource",
    };
  }

  // Get API path metadata from domain.extra
  const insertApiPathJson = _.get(
    component.properties,
    ["domain", "extra", "insertApiPath"],
    "",
  );

  if (!insertApiPathJson) {
    return {
      status: "error",
      message: "No insert API path metadata found - this resource may not support creation",
    };
  }

  const insertApiPath = JSON.parse(insertApiPathJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");

  // Get the get API path to determine how to extract resourceId later
  // APIs using {+name} need the full path; APIs using {name} need short name
  const getApiPathJson = _.get(
    component.properties,
    ["domain", "extra", "getApiPath"],
    "",
  );
  const getApiPath = getApiPathJson ? JSON.parse(getApiPathJson) : null;
  const usesFullResourcePath = getApiPath?.path?.includes("{+");

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  // Build the URL by replacing path parameters
  let url = `${baseUrl}${insertApiPath.path}`;
  const queryParams: string[] = [];

  // Replace path parameters with values from resource_value or domain
  // Parameters not found in the path template are added as query parameters
  if (insertApiPath.parameterOrder) {
    for (const paramName of insertApiPath.parameterOrder) {
      let paramValue;

      // Use extracted project_id for project parameter
      if (paramName === "project") {
        paramValue = projectId;
      } else if (paramName === "parent") {
        paramValue = _.get(component.properties, ["domain", "parent"]);
        if (!paramValue && projectId) {
          // auto-construct for project-only resources
          const availableScopesJson = _.get(component.properties, ["domain", "extra", "availableScopes"]);
          const availableScopes = availableScopesJson ? JSON.parse(availableScopesJson) : [];
          const isProjectOnly = availableScopes.length === 1 && availableScopes[0] === "projects";

          if (isProjectOnly) {
            const location = _.get(component.properties, ["domain", "location"]) ||
                            _.get(component.properties, ["domain", "zone"]) ||
                            _.get(component.properties, ["domain", "region"]);
            if (location) {
              paramValue = `projects/${projectId}/locations/${location}`;
            }
          }
        }
      } else {
        paramValue = _.get(component.properties, ["domain", paramName]);
      }

      if (paramValue) {
        // Handle {+param} (reserved expansion - don't encode, allows slashes)
        if (url.includes(`{+${paramName}}`)) {
          url = url.replace(`{+${paramName}}`, paramValue);
        } else if (url.includes(`{${paramName}}`)) {
          // Handle {param} (simple expansion - encode)
          url = url.replace(`{${paramName}}`, encodeURIComponent(paramValue));
        } else {
          // Parameter not in path template - add as query parameter
          // This handles APIs like GCS where project is a query param, not a path param
          queryParams.push(`${paramName}=${encodeURIComponent(paramValue)}`);
        }
      }
    }
  }

  // Append query parameters to URL
  if (queryParams.length > 0) {
    url += (url.includes("?") ? "&" : "?") + queryParams.join("&");
  }

  // Make the API request with retry logic
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "POST", // insert is always POST
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: codeString,
    });

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`Unable to create resource;
Called "${url}"
API returned ${resp.status} ${resp.statusText}:
${errorText}`
      ) as any;
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
    console.log(`[CREATE] LRO detected, polling for completion...`);

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
      isCompleteFn: (response, body) => body.status === "DONE" || body.done === true,
      isErrorFn: (response, body) => !!body.error,
      extractResultFn: async (response, body) => {
        // If operation has error, throw it
        if (body.error) {
          throw new Error(`Operation failed: ${JSON.stringify(body.error)}`);
        }

        // For create operations, get the final resource from the operation response
        // Some operations include the created resource in the response field
        if (body.response) {
          return body.response;
        }

        // GCP pattern: fetch the final resource from targetLink
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

        // Fallback: Try to extract resource name from operation metadata and fetch using getApiPath
        // This handles APIs that don't provide targetLink or response in the operation
        const operationMetadata = body.metadata;
        if (operationMetadata?.target) {
          const getApiPathJson = _.get(component.properties, ["domain", "extra", "getApiPath"], "");
          if (getApiPathJson) {
            const getApiPath = JSON.parse(getApiPathJson);
            let getUrl = `${baseUrl}${getApiPath.path}`;

            // Replace {+name} or {name} with the target resource name
            if (getUrl.includes("{+name}")) {
              getUrl = getUrl.replace("{+name}", operationMetadata.target);
            } else if (getUrl.includes("{name}")) {
              getUrl = getUrl.replace("{name}", encodeURIComponent(operationMetadata.target));
            }

            const resourceResponse = await fetch(getUrl, {
              method: "GET",
              headers: { "Authorization": `Bearer ${token}` },
            });

            if (resourceResponse.ok) {
              return await resourceResponse.json();
            }
          }
        }

        console.warn("[GCP] Operation completed but couldn't fetch final resource");
        return body;
      }
    });

    // Extract resource ID from the final resource
    // For GKE and similar APIs using {+name}, we need the full resource path
    // For Compute Engine style APIs using {name}, we need just the short name
    const resourceId = extractResourceId(finalResource, usesFullResourcePath);

    console.log(`[CREATE] Operation complete, resourceId: ${resourceId}`);
    return {
      resourceId: resourceId ? resourceId.toString() : undefined,
      status: "ok",
      payload: normalizeGcpResourceValues(finalResource),
    };
  }

  // Handle synchronous response
  const resourceId = extractResourceId(responseJson, usesFullResourcePath);

  if (resourceId) {
    return {
      resourceId: resourceId.toString(),
      status: "ok",
      payload: normalizeGcpResourceValues(responseJson),
    };
  } else {
    return {
      status: "ok",
      payload: normalizeGcpResourceValues(responseJson),
    };
  }
}

// Extract the resource ID from a GCP resource response
// For APIs using {+name} (like GKE), we need the full resource path from selfLink
// For APIs using {name} (like Compute Engine), we use the simple name/id
function extractResourceId(resource: any, useFullPath: boolean): string | undefined {
  // For APIs using {+name}, extract the full path from selfLink
  if (useFullPath && resource.selfLink && typeof resource.selfLink === "string") {
    try {
      const url = new URL(resource.selfLink);
      const pathParts = url.pathname.split("/").filter(Boolean);
      // Find "projects" and take everything from there
      const projectsIdx = pathParts.indexOf("projects");
      if (projectsIdx !== -1) {
        return pathParts.slice(projectsIdx).join("/");
      }
      // Fallback: skip the version (v1, v1beta1, etc.) and return the rest
      const versionIdx = pathParts.findIndex(p => /^v\d/.test(p));
      if (versionIdx !== -1 && versionIdx + 1 < pathParts.length) {
        return pathParts.slice(versionIdx + 1).join("/");
      }
    } catch {
      // If URL parsing fails, fall through to name/id
    }
  }

  // For Compute Engine style APIs or fallback, use simple name/id
  return resource.name || resource.id;
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
