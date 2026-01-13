async function main({ thisComponent }: Input): Promise<Output> {
  const component = thisComponent.properties;
  const resourcePayload = _.get(component, ["resource", "payload"], "");
  let resourceId = _.get(component, ["si", "resourceId"]);

  if (!resourceId) {
    return {
      status: "error",
      message: "No resourceId set, cannot import resource",
    };
  }

  // Get API path metadata from domain.extra
  const getApiPathJson = _.get(
    component,
    ["domain", "extra", "getApiPath"],
    "",
  );

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

  // Build the URL using shared utility
  const url = buildUrlWithParams(baseUrl, getApiPath, thisComponent, projectId, { resourceId });

  // Make the API request using shared authenticated GET
  let response;
  try {
    response = await authenticatedGet(url, token);
  } catch (error: any) {
    console.log("Failed to import Google Cloud resource");
    console.error(error.message || error);
    throw error;
  }

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
      self: {
        properties,
      },
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
