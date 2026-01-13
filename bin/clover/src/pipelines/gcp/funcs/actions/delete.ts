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

  // Use the stored httpMethod (some APIs like Service Networking use POST for delete)
  const httpMethod = deleteApiPath.httpMethod || "DELETE";

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

  // Build the URL using shared utility
  const url = buildUrlWithParams(baseUrl, deleteApiPath, component, projectId, { resourceId });

  // Make the API request with retry logic
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: httpMethod, // Usually DELETE, but some APIs use POST (e.g., deleteConnection)
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

      const error = new Error(`Unable to delete resource;
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

  // Handle 404 as success for delete operations
  if (response.status === 404) {
    return {
      status: "ok",
    };
  }

  // Handle 204 No Content (common for successful deletes like GCS)
  if (response.status === 204) {
    return {
      status: "ok",
    };
  }

  // Try to parse response body - some APIs return empty body on success
  const responseText = await response.text();
  if (!responseText) {
    return {
      status: "ok",
    };
  }

  const responseJson = JSON.parse(responseText);

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
