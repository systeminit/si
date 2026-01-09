async function main({ thisComponent }: Input): Promise<Output> {
  const component = thisComponent;

  // Get API path metadata from domain.extra
  const listApiPathJson = _.get(component.properties, ["domain", "extra", "listApiPath"], "");
  if (!listApiPathJson) {
    return {
      status: "error",
      message: "No list API path metadata found - this resource may not support discovery",
    };
  }

  const getApiPathJson = _.get(component.properties, ["domain", "extra", "getApiPath"], "");
  if (!getApiPathJson) {
    return {
      status: "error",
      message: "No get API path metadata found - this resource may not support discovery",
    };
  }

  const listApiPath = JSON.parse(listApiPathJson);
  const getApiPath = JSON.parse(getApiPathJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");
  const gcpResourceType = _.get(component.properties, ["domain", "extra", "GcpResourceType"], "");

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  console.log(`Discovering ${gcpResourceType} resources...`);

  // Build refinement filter from domain properties
  const refinement = buildRefinementFilter(thisComponent.properties.domain);

  // Build list URL and fetch all resources with pagination
  const listUrl = buildListUrl(baseUrl, listApiPath, component, projectId);
  const resources = await fetchAllResources(listUrl, token);

  console.log(`Found ${resources.length} resources`);

  // Process each resource
  const create: Output["ops"]["create"] = {};
  const actions: Record<string, any> = {};
  let importCount = 0;

  for (const resource of resources) {
    const resourceId = resource.name || resource.id || resource.selfLink;
    if (!resourceId) {
      console.log(`Skipping resource without ID`);
      continue;
    }

    console.log(`Importing ${resourceId}`);

    // Fetch full resource details
    const fullResource = await fetchResourceDetails(
      baseUrl, getApiPath, component, projectId, token, resource, resourceId
    );

    if (!fullResource) {
      console.log(`Failed to fetch ${resourceId}, skipping`);
      continue;
    }

    const properties = {
      si: { resourceId },
      domain: {
        ...component.properties?.domain || {},
        ...fullResource,
      },
      resource: fullResource,
    };

    // Apply refinement filter
    if (_.isEmpty(refinement) || _.isMatch(properties.domain, refinement)) {
      const newAttributes: Output["ops"]["create"][string]["attributes"] = {};
      for (const [skey, svalue] of Object.entries(component.sources || {})) {
        newAttributes[skey] = { $source: svalue };
      }

      create[resourceId] = {
        kind: gcpResourceType || component.properties?.domain?.extra?.GcpResourceType,
        properties,
        attributes: newAttributes,
      };
      actions[resourceId] = { remove: ["create"] };
      importCount++;
    } else {
      console.log(`Skipping import of ${resourceId}; it did not match refinements`);
    }
  }

  return {
    status: "ok",
    message: `Discovered ${importCount} ${gcpResourceType} resources`,
    ops: { create, actions },
  };
}

// ============================================================================
// Helper Functions
// ============================================================================

// Build refinement filter from domain properties
function buildRefinementFilter(domain: any): Record<string, any> {
  const refinement = _.cloneDeep(domain);
  delete refinement["extra"];

  for (const [key, value] of Object.entries(refinement)) {
    if (_.isEmpty(value)) {
      delete refinement[key];
    } else if (_.isPlainObject(value)) {
      refinement[key] = _.pickBy(
        value as Record<string, any>,
        (v) => !_.isEmpty(v) || _.isNumber(v) || _.isBoolean(v),
      );
      if (_.isEmpty(refinement[key])) {
        delete refinement[key];
      }
    }
  }

  return refinement;
}

// Get location from component
function getLocation(component: any): string | undefined {
  return _.get(component.properties, ["domain", "location"]) ||
    _.get(component.properties, ["domain", "zone"]) ||
    _.get(component.properties, ["domain", "region"]);
}

// Resolve parameter value for list operations (discovery)
function resolveListParamValue(
  component: any,
  paramName: string,
  projectId: string | undefined
): string | undefined {
  if (paramName === "project" || paramName === "projectId") {
    return projectId;
  }

  if (paramName === "parent") {
    let parentValue = _.get(component.properties, ["domain", "parent"]);
    if (!parentValue && projectId) {
      const location = getLocation(component);
      // For discovery, always auto-construct parent (fallback to project-only)
      parentValue = location
        ? `projects/${projectId}/locations/${location}`
        : `projects/${projectId}`;
    }
    return parentValue;
  }

  return _.get(component.properties, ["domain", paramName]);
}

// Build list URL by replacing path parameters
function buildListUrl(
  baseUrl: string,
  listApiPath: { path: string; parameterOrder?: string[] },
  component: any,
  projectId: string | undefined
): string {
  let url = `${baseUrl}${listApiPath.path}`;
  const queryParams: string[] = [];

  if (listApiPath.parameterOrder) {
    for (const paramName of listApiPath.parameterOrder) {
      const paramValue = resolveListParamValue(component, paramName, projectId);

      if (paramValue) {
        if (url.includes(`{+${paramName}}`)) {
          url = url.replace(`{+${paramName}}`, paramValue);
        } else if (url.includes(`{${paramName}}`)) {
          url = url.replace(`{${paramName}}`, encodeURIComponent(paramValue));
        }
      }
    }
  }

  // Handle parent as query parameter for some APIs
  if (!url.includes("parent=") && !listApiPath.path.includes("{parent}") && !listApiPath.path.includes("{+parent}")) {
    const parentValue = _.get(component.properties, ["domain", "parent"]);
    if (parentValue) {
      queryParams.push(`parent=${encodeURIComponent(parentValue)}`);
    }
  }

  if (queryParams.length > 0) {
    url += (url.includes("?") ? "&" : "?") + queryParams.join("&");
  }

  return url;
}

// Fetch all resources with pagination
async function fetchAllResources(listUrl: string, token: string): Promise<any[]> {
  let resources: any[] = [];
  let nextPageToken: string | null = null;

  do {
    let currentUrl = listUrl;
    if (nextPageToken) {
      currentUrl += (listUrl.includes("?") ? "&" : "?") + `pageToken=${encodeURIComponent(nextPageToken)}`;
    }

    const response = await siExec.withRetry(async () => {
      const resp = await fetch(currentUrl, {
        method: "GET",
        headers: { "Authorization": `Bearer ${token}` },
      });

      if (!resp.ok) {
        const errorText = await resp.text();
        const error = new Error(`Google Cloud API Error: ${resp.status} ${resp.statusText} - ${errorText}`) as any;
        error.status = resp.status;
        error.body = errorText;
        throw error;
      }

      return resp;
    }, {
      isRateLimitedFn: (error: any) => error.status === 429
    }).then((r: any) => r.result);

    const listData = await response.json();

    // GCP list responses vary - find the array containing resources
    let items = listData.items;
    if (!items) {
      for (const [key, value] of Object.entries(listData)) {
        if (Array.isArray(value) && key !== "unreachable" && key !== "warnings") {
          items = value;
          break;
        }
      }
    }

    resources = resources.concat(items || []);
    nextPageToken = listData.nextPageToken || null;

    if (nextPageToken) {
      console.log(`Fetching next page...`);
    }
  } while (nextPageToken);

  return resources;
}

// Fetch full resource details
async function fetchResourceDetails(
  baseUrl: string,
  getApiPath: { path: string; parameterOrder?: string[] },
  component: any,
  projectId: string | undefined,
  token: string,
  resource: any,
  resourceId: string
): Promise<any | null> {
  let url = `${baseUrl}${getApiPath.path}`;

  if (getApiPath.parameterOrder) {
    const lastParam = getApiPath.parameterOrder[getApiPath.parameterOrder.length - 1];

    for (const paramName of getApiPath.parameterOrder) {
      let paramValue: string | undefined;

      if (paramName === lastParam) {
        paramValue = resourceId;
      } else if (paramName === "project" || paramName === "projectId") {
        paramValue = projectId;
      } else {
        paramValue = _.get(resource, [paramName]) ||
          _.get(component.properties, ["domain", paramName]);
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

  try {
    const response = await siExec.withRetry(async () => {
      const resp = await fetch(url, {
        method: "GET",
        headers: { "Authorization": `Bearer ${token}` },
      });

      if (!resp.ok) {
        const error = new Error(`Failed to fetch ${resourceId}`) as any;
        error.status = resp.status;
        throw error;
      }

      return resp;
    }, {
      isRateLimitedFn: (error: any) => error.status === 429
    }).then((r: any) => r.result);

    return await response.json();
  } catch {
    return null;
  }
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
