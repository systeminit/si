async function main(component: Input): Promise<Output> {
  // Get API path metadata from domain.extra
  const deleteApiPathJson = _.get(
    component.properties,
    ["domain", "extra", "deleteApiPath"],
    "",
  );

  if (!deleteApiPathJson) {
    return {
      status: "error",
      message: "No delete API path metadata found - this resource may not support deletion",
    };
  }

  const deleteApiPath = JSON.parse(deleteApiPathJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");

  // Get resourceId
  const resourceId = component.properties?.si?.resourceId;
  if (!resourceId) {
    return {
      status: "error",
      message: "No resource ID found for deletion",
    };
  }

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  // Build the URL by replacing path parameters
  let url = `${baseUrl}${deleteApiPath.path}`;

  // Replace path parameters with values from resource_value or domain
  if (deleteApiPath.parameterOrder) {
    for (const paramName of deleteApiPath.parameterOrder) {
      let paramValue;

      // For the resource identifier, use resourceId
      if (paramName === deleteApiPath.parameterOrder[deleteApiPath.parameterOrder.length - 1]) {
        paramValue = resourceId;
      } else if (paramName === "project") {
        // Use extracted project_id for project parameter
        paramValue = projectId;
      } else {
        paramValue = _.get(component.properties, ["resource", "payload", paramName]) ||
                     _.get(component.properties, ["domain", paramName]);

        // GCP often returns full URLs for reference fields e.g.
        // region: //www.googleapis.com/compute/v1/projects/myproject/regions/us-central1
        // network: //www.googleapis.com/compute/v1/projects/myproject/networks/my-network

        // Extract just the resource name from the URL
        if (paramValue && typeof paramValue === "string" && paramValue.startsWith("https://")) {
          const urlParts = paramValue.split("/");
          paramValue = urlParts[urlParts.length - 1];
        }
      }

      if (paramValue) {
        url = url.replace(`{${paramName}}`, encodeURIComponent(paramValue));
      }
    }
  }

  // Make the API request
  const response = await fetch(url, {
    method: "DELETE", // delete is always DELETE
    headers: {
      "Authorization": `Bearer ${token}`,
    },
  });

  if (!response.ok) {
    // If already deleted (404), consider it success
    if (response.status === 404) {
      return {
        status: "ok",
      };
    }

    const errorText = await response.text();
    return {
      status: "error",
      message: `Unable to delete resource; API returned ${response.status} ${response.statusText}: ${errorText}`,
    };
  }

  return {
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
