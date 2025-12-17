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

  // Build the URL by replacing path parameters
  let url = `${baseUrl}${getApiPath.path}`;

  // Replace path parameters with values from resource_value or domain
  if (getApiPath.parameterOrder) {
    for (const paramName of getApiPath.parameterOrder) {
      let paramValue;

      // For the resource identifier, use resourceId
      if (paramName === getApiPath.parameterOrder[getApiPath.parameterOrder.length - 1]) {
        paramValue = resourceId;
      } else if (paramName === "project") {
        // Use extracted project_id for project parameter
        paramValue = projectId;
      } else {
        paramValue = _.get(component, ["domain", paramName]);
      }

      if (paramValue) {
        url = url.replace(`{${paramName}}`, encodeURIComponent(paramValue));
      }
    }
  }

  // Make the API request
  const response = await fetch(url, {
    method: "GET",
    headers: {
      "Authorization": `Bearer ${token}`,
    },
  });

  if (!response.ok) {
    const errorText = await response.text();
    console.log("Failed to import Google Cloud resource");
    console.error(errorText);
    return {
      status: "error",
      message: `Import error; API returned ${response.status} ${response.statusText}: ${errorText}`,
    };
  }

  const resourceProperties = await response.json();
  console.log(resourceProperties);

  const properties = {
    ...component,
    domain: {
      ...component.domain,
      ...resourceProperties,
    },
  };

  let needsRefresh = true;
  if (!resourcePayload) {
    properties.resource = resourceProperties;
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
