async function main({ thisComponent }: Input): Promise<Output> {
  const component = thisComponent;

  // Get API path metadata from domain.extra
  const listApiPathJson = _.get(
    component.properties,
    ["domain", "extra", "listApiPath"],
    "",
  );

  if (!listApiPathJson) {
    return {
      status: "error",
      message: "No list API path metadata found - this resource may not support discovery",
    };
  }

  const listApiPath = JSON.parse(listApiPathJson);
  const getApiPathJson = _.get(
    component.properties,
    ["domain", "extra", "getApiPath"],
    "",
  );

  if (!getApiPathJson) {
    return {
      status: "error",
      message: "No get API path metadata found - this resource may not support discovery",
    };
  }

  const getApiPath = JSON.parse(getApiPathJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");
  const gcpResourceType = _.get(
    component.properties,
    ["domain", "extra", "GcpResourceType"],
    "",
  );

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  console.log(`Discovering ${gcpResourceType} resources...`);

  // Build refinement filter from domain properties
  const refinement = _.cloneDeep(thisComponent.properties.domain);
  delete refinement["extra"];
  // Remove any empty values, as they are never refinements
  for (const [key, value] of Object.entries(refinement)) {
    if (_.isEmpty(value)) {
      delete refinement[key];
    } else if (_.isPlainObject(value)) {
      refinement[key] = _.pickBy(
        value,
        (v) => !_.isEmpty(v) || _.isNumber(v) || _.isBoolean(v),
      );
      if (_.isEmpty(refinement[key])) {
        delete refinement[key];
      }
    }
  }

  // Build list URL using shared utility (forList=true for discovery)
  let listUrl = buildUrlWithParams(baseUrl, listApiPath, component, projectId, { forList: true });

  // Handle parent as query parameter for APIs like Resource Manager Folders
  // that don't use parent in the path but require it as a query parameter
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

  // Handle pagination with pageToken
  let resources: any[] = [];
  let nextPageToken: string | null = null;

  do {
    let currentUrl = listUrl;
    if (nextPageToken) {
      const separator = listUrl.includes("?") ? "&" : "?";
      currentUrl = `${listUrl}${separator}pageToken=${encodeURIComponent(nextPageToken)}`;
    }

    const listResponse = await authenticatedGet(currentUrl, token);
    const listData = await listResponse.json();

    // GCP list responses vary in structure:
    // - Compute Engine uses "items" array
    // - Other APIs use the plural resource name (e.g., "contacts", "clusters", "buckets")
    // Try "items" first, then look for any array property that isn't metadata
    let items = listData.items;
    if (!items) {
      // Find the first array property that likely contains resources
      for (const [key, value] of Object.entries(listData)) {
        if (Array.isArray(value) && key !== "unreachable" && key !== "warnings") {
          items = value;
          break;
        }
      }
    }
    items = items || [];
    resources = resources.concat(items);
    nextPageToken = listData.nextPageToken || null;

    if (nextPageToken) {
      console.log(`Fetching next page...`);
    }
  } while (nextPageToken);

  console.log(`Found ${resources.length} resources`);

  const create: Output["ops"]["create"] = {};
  const actions = {};
  let importCount = 0;

  for (const resource of resources) {
    // The resource ID is typically in the "id" or "name" field
    const resourceId = resource.id || resource.name || resource.selfLink;

    if (!resourceId) {
      console.log(`Skipping resource without ID`);
      continue;
    }

    console.log(`Importing ${resourceId}`);

    // Build the get URL to fetch full resource details
    // Note: We can't use buildUrlWithParams here because we need to pull
    // some parameters from the discovered resource, not from component.properties
    let getUrl = `${baseUrl}${getApiPath.path}`;

    if (getApiPath.parameterOrder) {
      for (const paramName of getApiPath.parameterOrder) {
        let paramValue;

        // For the resource identifier, use resourceId
        if (paramName === getApiPath.parameterOrder[getApiPath.parameterOrder.length - 1]) {
          paramValue = resourceId;
        } else if (paramName === "project" || paramName === "projectId") {
          paramValue = projectId;
        } else {
          // Try to get from discovered resource first, then fall back to domain
          paramValue = _.get(resource, [paramName]) ||
            _.get(component.properties, ["domain", paramName]);

          // GCP often returns full URLs for reference fields - extract just the resource name
          if (paramValue && typeof paramValue === "string" && paramValue.startsWith("https://")) {
            const urlParts = paramValue.split("/");
            paramValue = urlParts[urlParts.length - 1];
          }
        }

        if (paramValue) {
          // {+param} = reserved expansion (no encoding, allows slashes)
          // {param} = simple expansion (URL encoded)
          if (getUrl.includes(`{+${paramName}}`)) {
            getUrl = getUrl.replace(`{+${paramName}}`, paramValue);
          } else if (getUrl.includes(`{${paramName}}`)) {
            getUrl = getUrl.replace(`{${paramName}}`, encodeURIComponent(paramValue));
          }
        }
      }
    }

    // Fetch the full resource details with retry
    let resourceResponse;
    try {
      resourceResponse = await authenticatedGet(getUrl, token);
    } catch (error) {
      console.log(`Failed to fetch ${resourceId} after retries, skipping`);
      continue;
    }

    const fullResource = await resourceResponse.json();

    // Normalize the resource to convert full GCP URLs to simple values
    const normalizedResource = normalizeGcpResourceValues(fullResource);

    const properties = {
      si: {
        resourceId,
      },
      domain: {
        ...component.properties?.domain || {},
        ...normalizedResource,
      },
      resource: normalizedResource,
    };

    // Apply refinement filter
    if (_.isEmpty(refinement) || _.isMatch(properties.domain, refinement)) {
      const newAttributes: Output["ops"]["create"][string]["attributes"] = {};
      for (const [skey, svalue] of Object.entries(component.sources || {})) {
        newAttributes[skey] = {
          $source: svalue,
        };
      }

      create[resourceId] = {
        kind: gcpResourceType || component.properties?.domain?.extra?.GcpResourceType,
        properties,
        attributes: newAttributes,
      };
      actions[resourceId] = {
        remove: ["create"],
      };
      importCount++;
    } else {
      console.log(
        `Skipping import of ${resourceId}; it did not match refinements`,
      );
    }
  }

  return {
    status: "ok",
    message: `Discovered ${importCount} ${gcpResourceType} resources`,
    ops: {
      create,
      actions,
    },
  };
}
