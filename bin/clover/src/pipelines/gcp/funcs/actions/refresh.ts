async function main(component: Input): Promise<Output> {
  const isListOnly = _.get(component.properties, ["domain", "extra", "listOnly"]) === "true";

  const resourceId = component.properties?.si?.resourceId;
  if (!resourceId) {
    return { status: "error", message: "No resource ID found for refresh" };
  }

  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");

  if (isListOnly) {
    return refreshViaList(component, resourceId, token, projectId, baseUrl);
  }
  return refreshViaGet(component, resourceId, token, projectId, baseUrl);
}

// Resolve a parameter value from component properties
// forList: controls parent auto-construction behavior
//   - true (list operations): always auto-construct parent, fallback to projects/${projectId}
//   - false (get operations): only auto-construct if supportsParentAutoConstruct is true
function resolveParamValue(
  component: Input,
  paramName: string,
  projectId: string | undefined,
  forList: boolean = false
): string | undefined {
  if (paramName === "project" || paramName === "projectId") {
    return projectId;
  }

  if (paramName === "parent") {
    let parentValue = _.get(component.properties, ["resource", "payload", "parent"]) ||
      _.get(component.properties, ["domain", "parent"]);
    if (!parentValue && projectId) {
      const location = getLocation(component);
      const supportsAutoConstruct = _.get(component.properties, ["domain", "extra", "supportsParentAutoConstruct"]) === "true";

      if (forList) {
        // List operations: always auto-construct, fallback to project-only
        parentValue = location ? `projects/${projectId}/locations/${location}` : `projects/${projectId}`;
      } else if (supportsAutoConstruct && location) {
        // Get/update/delete operations: only auto-construct if metadata says we can
        parentValue = `projects/${projectId}/locations/${location}`;
      }
    }
    return parentValue;
  }

  let paramValue = _.get(component.properties, ["resource", "payload", paramName]) ||
    _.get(component.properties, ["domain", paramName]);

  // GCP often returns full URLs for reference fields - extract just the resource name
  if (paramValue && typeof paramValue === "string" && paramValue.startsWith("https://")) {
    const urlParts = paramValue.split("/");
    paramValue = urlParts[urlParts.length - 1];
  }

  return paramValue;
}

// Get location from component, checking resource payload first then domain
function getLocation(component: Input): string | undefined {
  return _.get(component.properties, ["resource", "payload", "location"]) ||
    _.get(component.properties, ["domain", "location"]) ||
    _.get(component.properties, ["domain", "zone"]) ||
    _.get(component.properties, ["domain", "region"]);
}

// Check if resourceId is already a full path matching the API path structure
// Uses proper segment matching (not substring) to avoid false positives
function isFullResourcePath(resourceId: string, pathTemplate: string): boolean {
  if (!resourceId.includes('/')) return false;

  // Extract static segments from template (non-parameter parts)
  // e.g., "projects/{+projectId}/datasets/{+datasetId}/tables/{+tableId}" -> ["projects", "datasets", "tables"]
  const templateSegments = pathTemplate.split('/').filter(s => !s.startsWith('{'));

  // Check if resourceId contains these as actual path segments (bounded by /)
  // This prevents false positives like "my-projects-datasets" matching "projects/datasets"
  const resourceSegments = resourceId.split('/');
  let templateIdx = 0;

  for (const seg of resourceSegments) {
    if (templateIdx < templateSegments.length && seg === templateSegments[templateIdx]) {
      templateIdx++;
    }
  }

  return templateIdx === templateSegments.length;
}

// Build URL by replacing path parameters using RFC 6570 URI templates
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
        }
      }
    }
  }

  return url;
}

// Make authenticated GET request with retry logic
async function authenticatedGet(url: string, token: string, allow404 = false): Promise<Response> {
  return siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "GET",
      headers: { "Authorization": `Bearer ${token}` },
    });

    if (!resp.ok) {
      if (allow404 && resp.status === 404) {
        return resp;
      }
      const errorText = await resp.text();

      const error = new Error(`Unable to refresh resource;
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
}

async function refreshViaGet(
  component: Input,
  resourceId: string,
  token: string,
  projectId: string | undefined,
  baseUrl: string
): Promise<Output> {
  const getApiPathJson = _.get(component.properties, ["domain", "extra", "getApiPath"], "");
  if (!getApiPathJson) {
    return { status: "error", message: "No get API path metadata found - this resource may not support refresh" };
  }

  const getApiPath = JSON.parse(getApiPathJson);
  const url = buildUrlWithParams(baseUrl, getApiPath, component, projectId, { resourceId });
  const response = await authenticatedGet(url, token, true);

  if (response.status === 404) {
    return { status: "ok", payload: null };
  }

  const responseJson = await response.json();
  return { payload: normalizeGcpResourceValues(responseJson), status: "ok" };
}

async function refreshViaList(
  component: Input,
  resourceId: string,
  token: string,
  projectId: string | undefined,
  baseUrl: string
): Promise<Output> {
  const listApiPathJson = _.get(component.properties, ["domain", "extra", "listApiPath"], "");
  if (!listApiPathJson) {
    return { status: "error", message: "No list API path metadata found - this resource may not support refresh" };
  }

  const listApiPath = JSON.parse(listApiPathJson);
  let listUrl = buildUrlWithParams(baseUrl, listApiPath, component, projectId, { forList: true });

  // Handle parent as query parameter for some APIs
  if (!listUrl.includes("parent=") && !listApiPath.path.includes("{parent}") && !listApiPath.path.includes("{+parent}")) {
    const parentValue = resolveParamValue(component, "parent", projectId, true);
    if (parentValue) {
      listUrl += (listUrl.includes("?") ? "&" : "?") + `parent=${encodeURIComponent(parentValue)}`;
    }
  }

  // Paginate through list results to find matching resource
  let nextPageToken: string | null = null;

  do {
    let paginatedUrl = listUrl;
    if (nextPageToken) {
      paginatedUrl += (paginatedUrl.includes("?") ? "&" : "?") + `pageToken=${encodeURIComponent(nextPageToken)}`;
    }

    const response = await authenticatedGet(paginatedUrl, token);
    const listData = await response.json();

    // GCP list responses vary - find the array property containing resources
    let items = listData.items;
    if (!items) {
      for (const [key, value] of Object.entries(listData)) {
        if (Array.isArray(value) && key !== "unreachable" && key !== "warnings") {
          items = value;
          break;
        }
      }
    }

    // Find resource matching our resourceId
    for (const resource of items || []) {
      const resourceName = resource.name || resource.id;
      if (resourceName === resourceId || resourceName?.endsWith(`/${resourceId}`) || resourceId.endsWith(`/${resourceName}`)) {
        return { payload: normalizeGcpResourceValues(resource), status: "ok" };
      }
    }

    nextPageToken = listData.nextPageToken || null;
  } while (nextPageToken);

  // Resource not found - may have been deleted
  console.log(`[REFRESH] Resource ${resourceId} not found in list results - may have been deleted`);
  return { status: "ok", payload: null };
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
