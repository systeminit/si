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
  // Only add parent if it's declared as a valid query parameter in the API
  if (!listUrl.includes("parent=") && !listApiPath.path.includes("{parent}") && !listApiPath.path.includes("{+parent}")) {
    const validQueryParams = listApiPath.queryParams || [];
    if (validQueryParams.includes("parent")) {
      const parentValue = resolveParamValue(component, "parent", projectId, true);
      if (parentValue) {
        listUrl += (listUrl.includes("?") ? "&" : "?") + `parent=${encodeURIComponent(parentValue)}`;
      }
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

    // Find resource matching our resourceId or domain.name (fallback for SQL Users, etc.)
    const domainName = _.get(component.properties, ["domain", "name"]);
    for (const resource of items || []) {
      const resourceName = resource.name || resource.id;
      if (resourceName === resourceId || resourceName === domainName ||
        resourceName?.endsWith(`/${resourceId}`) || resourceId.endsWith(`/${resourceName}`)) {
        return { payload: normalizeGcpResourceValues(resource), status: "ok" };
      }
    }

    nextPageToken = listData.nextPageToken || null;
  } while (nextPageToken);

  // Resource not found - may have been deleted
  console.log(`[REFRESH] Resource ${resourceId} not found in list results - may have been deleted`);
  return { status: "ok", payload: null };
}
