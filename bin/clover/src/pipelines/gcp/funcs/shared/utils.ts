// Shared utilities for GCP functions
// These functions are automatically prepended to each func file during spec generation

// URL normalization for GCP resource values
const GCP_URL_PATTERN = /^https:\/\/[^/]*\.?googleapis\.com\//;
const LOCATION_SEGMENTS = new Set(["regions", "zones", "locations"]);

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

function getLocation(component: Input): string | undefined {
  return _.get(component.properties, ["domain", "location"]) ||
    _.get(component.properties, ["domain", "zone"]) ||
    _.get(component.properties, ["domain", "region"]);
}

function resolveParamValue(
  component: Input,
  paramName: string,
  projectId: string | undefined,
  forList = false
): string | undefined {
  // Use extracted project_id for project/projectId parameter
  if (paramName === "project" || paramName === "projectId") {
    return projectId;
  }

  if (paramName === "parent") {
    // Check resource.payload first (for existing resources), then domain
    let parentValue = _.get(component.properties, ["resource", "payload", "parent"]) ||
      _.get(component.properties, ["domain", "parent"]);
    if (!parentValue && projectId) {
      const location = _.get(component.properties, ["resource", "payload", "location"]) ||
        _.get(component.properties, ["domain", "location"]) ||
        _.get(component.properties, ["domain", "zone"]) ||
        _.get(component.properties, ["domain", "region"]);

      if (forList) {
        // List operations: always auto-construct, fallback to project-only
        parentValue = location ? `projects/${projectId}/locations/${location}` : `projects/${projectId}`;
      } else {
        // Get/Update/Delete operations: only auto-construct for project-only resources
        const availableScopesJson = _.get(component.properties, ["domain", "extra", "availableScopes"]);
        const availableScopes = availableScopesJson ? JSON.parse(availableScopesJson) : [];
        const isProjectOnly = availableScopes.length === 1 && availableScopes[0] === "projects";

        if (isProjectOnly && location) {
          parentValue = `projects/${projectId}/locations/${location}`;
        }
      }
    }
    return parentValue;
  }

  // Check resource.payload first (for existing resources), then domain
  let paramValue = _.get(component.properties, ["resource", "payload", paramName]) ||
    _.get(component.properties, ["domain", paramName]);

  // GCP often returns full URLs for reference fields - extract just the resource name
  if (paramValue && typeof paramValue === "string" && paramValue.startsWith("https://")) {
    const urlParts = paramValue.split("/");
    paramValue = urlParts[urlParts.length - 1];
  }

  return paramValue;
}

function isFullResourcePath(resourceId: string, pathTemplate: string): boolean {
  // Count path segments in the template (excluding empty strings)
  const templateSegments = pathTemplate.split("/").filter(seg => seg && !seg.includes("{"));

  // Count matching segments in the resourceId
  const resourceSegments = resourceId.split("/").filter(Boolean);

  if (resourceSegments.length < templateSegments.length) {
    return false;
  }

  let templateIdx = 0;
  for (const seg of resourceSegments) {
    if (templateIdx < templateSegments.length && seg === templateSegments[templateIdx]) {
      templateIdx++;
    }
  }

  return templateIdx === templateSegments.length;
}

function buildUrlWithParams(
  baseUrl: string,
  apiPath: { path: string; parameterOrder?: string[] },
  component: Input,
  projectId: string | undefined,
  options: { resourceId?: string; forList?: boolean } = {}
): string {
  // If resourceId is already a full path matching the API structure, use it directly
  // This handles cases where resourceId is "projects/xxx/datasets/yyy/tables/zzz"
  if (options.resourceId && isFullResourcePath(options.resourceId, apiPath.path)) {
    return `${baseUrl}${options.resourceId}`;
  }

  let url = `${baseUrl}${apiPath.path}`;
  const queryParams: string[] = [];

  if (apiPath.parameterOrder) {
    const lastParam = apiPath.parameterOrder[apiPath.parameterOrder.length - 1];

    for (const paramName of apiPath.parameterOrder) {
      let paramValue: string | undefined;

      // For get requests, use resourceId for the last parameter
      if (options.resourceId && paramName === lastParam) {
        paramValue = options.resourceId;
      } else {
        paramValue = resolveParamValue(component, paramName, projectId, options.forList);
      }

      if (paramValue) {
        // Handle {+param} (reserved expansion - don't encode, allows slashes)
        if (url.includes(`{+${paramName}}`)) {
          url = url.replace(`{+${paramName}}`, paramValue);
        } else if (url.includes(`{${paramName}}`)) {
          // Handle {param} (simple expansion - encode)
          url = url.replace(`{${paramName}}`, encodeURIComponent(paramValue));
        } else {
          // Parameter not in path template - add as query parameter (e.g., GCS project)
          queryParams.push(`${paramName}=${encodeURIComponent(paramValue)}`);
        }
      }
    }

    // Add query parameters if any
    if (queryParams.length > 0) {
      url += `?${queryParams.join("&")}`;
    }
  }

  // Fail fast if URL still contains unresolved placeholders
  // This catches cases where required parameters (like location in parent) are missing
  const unresolvedMatches = [...url.matchAll(/\{[+]?([^}]+)\}/g)];
  if (unresolvedMatches.length > 0) {
    const unresolvedParams = unresolvedMatches.map(m => m[1]);
    const paramList = unresolvedParams.join(", ");
    throw new Error(
      `Cannot build API URL - unresolved parameters: ${paramList}\n` +
      `URL template result: ${url}\n` +
      `Please set the required properties (${paramList}) before running this operation.`
    );
  }

  return url;
}

// Make authenticated GET request with retry logic
async function authenticatedGet(url: string, token: string, allow404 = false): Promise<Response> {
  const retryResult = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "GET",
      headers: { "Authorization": `Bearer ${token}` },
    });

    if (resp.status === 404 && allow404) {
      return resp;
    }

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`HTTP ${resp.status}: ${errorText}`) as any;
      error.status = resp.status;
      throw error;
    }

    return resp;
  }, {
    maxAttempts: 5,
    isRateLimitedFn: (error) => error.status === 429 || error.status === 503
  });
  return retryResult.result;
}

// Make authenticated POST request with retry logic
async function authenticatedPost(url: string, token: string, body?: string): Promise<Response> {
  const retryResult = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "POST",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json"
      },
      body: body,
    });

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`HTTP ${resp.status}: ${errorText}`) as any;
      error.status = resp.status;
      throw error;
    }

    return resp;
  }, {
    maxAttempts: 5,
    isRateLimitedFn: (error) => error.status === 429 || error.status === 503
  });
  return retryResult.result;
}

// Make authenticated PATCH request with retry logic
async function authenticatedPatch(url: string, token: string, body?: string): Promise<Response> {
  const retryResult = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "PATCH",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json"
      },
      body: body,
    });

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`HTTP ${resp.status}: ${errorText}`) as any;
      error.status = resp.status;
      throw error;
    }

    return resp;
  }, {
    maxAttempts: 5,
    isRateLimitedFn: (error) => error.status === 429 || error.status === 503
  });
  return retryResult.result;
}

// Make authenticated DELETE request with retry logic
async function authenticatedDelete(url: string, token: string): Promise<Response> {
  const retryResult = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "DELETE",
      headers: { "Authorization": `Bearer ${token}` },
    });

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`HTTP ${resp.status}: ${errorText}`) as any;
      error.status = resp.status;
      throw error;
    }

    return resp;
  }, {
    maxAttempts: 5,
    isRateLimitedFn: (error) => error.status === 429 || error.status === 503
  });
  return retryResult.result;
}
