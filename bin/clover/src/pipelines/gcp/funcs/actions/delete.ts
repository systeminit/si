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
  // GCP APIs use RFC 6570 URI templates: {param} and {+param} (reserved expansion)
  if (deleteApiPath.parameterOrder) {
    for (const paramName of deleteApiPath.parameterOrder) {
      let paramValue;

      // For the resource identifier, use resourceId
      if (paramName === deleteApiPath.parameterOrder[deleteApiPath.parameterOrder.length - 1]) {
        paramValue = resourceId;
      } else if (paramName === "project") {
        // Use extracted project_id for project parameter
        paramValue = projectId;
      } else if (paramName === "parent") {
        // "parent" is a common GCP pattern: projects/{project}/locations/{location}
        paramValue = _.get(component.properties, ["resource", "payload", "parent"]) ||
                     _.get(component.properties, ["domain", "parent"]);
        if (!paramValue && projectId) {
          const location = _.get(component.properties, ["resource", "payload", "location"]) ||
                          _.get(component.properties, ["domain", "location"]) ||
                          _.get(component.properties, ["domain", "zone"]) ||
                          _.get(component.properties, ["domain", "region"]);
          if (location) {
            paramValue = `projects/${projectId}/locations/${location}`;
          }
        }
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

  // Make the API request with retry logic
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "DELETE", // delete is always DELETE
      headers: {
        "Authorization": `Bearer ${token}`,
      },
    });

    if (!resp.ok) {
      // If already deleted (404), consider it success
      if (resp.status === 404) {
        return resp;
      }

      const errorText = await resp.text();
      const error = new Error(`Unable to delete resource; API returned ${resp.status} ${resp.statusText}: ${errorText}`) as any;
      error.status = resp.status;
      error.body = errorText;
      throw error;
    }

    return resp;
  }, {
    isRateLimitedFn: (error) => error.status === 429
  }).then((r) => r.result);

  // Handle 404 as success for delete operations
  if (response.status === 404) {
    return {
      status: "ok",
    };
  }

  const responseJson = await response.json();

  // Handle Google Cloud Long-Running Operations (LRO)
  // Check if this is an operation response:
  // - Compute Engine uses "kind" containing "operation"
  // - GKE/Container API uses "operationType" field
  const isLRO = (responseJson.kind && responseJson.kind.includes("operation")) ||
                responseJson.operationType;
  if (isLRO) {
    console.log(`[DELETE] LRO detected, polling for completion...`);

    // Use selfLink or construct URL from operation name
    const pollingUrl = responseJson.selfLink || `${baseUrl}${responseJson.name}`;

    // Poll the operation until it completes
    await siExec.pollLRO({
      url: pollingUrl,
      headers: { "Authorization": `Bearer ${token}` },
      maxAttempts: 20,
      baseDelay: 2000,
      maxDelay: 30000,
      isCompleteFn: (response, body) => body.status === "DONE",
      isErrorFn: (response, body) => !!body.error,
      extractResultFn: async (response, body) => {
        // If operation has error, throw it
        if (body.error) {
          throw new Error(`Delete operation failed: ${JSON.stringify(body.error)}`);
        }
        return body;
      }
    });

    console.log(`[DELETE] Operation complete`);
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
