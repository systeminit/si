async function main(component: Input): Promise<Output> {
  // Check if resource already exists
  const existingPayload = component.properties.resource?.payload;
  if (existingPayload) {
    return {
      status: "error",
      message: "Resource already exists",
      payload: existingPayload,
    };
  }

  // Get the generated code from code gen function
  const codeString = component.properties.code?.["Google Cloud Create Code Gen"]?.code;
  if (!codeString) {
    return {
      status: "error",
      message: "Could not find Google Cloud Create Code Gen code for resource",
    };
  }

  // Get API path metadata from domain.extra
  const insertApiPathJson = _.get(
    component.properties,
    ["domain", "extra", "insertApiPath"],
    "",
  );

  if (!insertApiPathJson) {
    return {
      status: "error",
      message: "No insert API path metadata found - this resource may not support creation",
    };
  }

  const insertApiPath = JSON.parse(insertApiPathJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  // Build the URL by replacing path parameters
  let url = `${baseUrl}${insertApiPath.path}`;

  // Replace path parameters with values from resource_value or domain
  if (insertApiPath.parameterOrder) {
    for (const paramName of insertApiPath.parameterOrder) {
      let paramValue;

      // Use extracted project_id for project parameter
      if (paramName === "project") {
        paramValue = projectId;
      } else {
        paramValue = _.get(component.properties, ["domain", paramName]);
      }

      if (paramValue) {
        url = url.replace(`{${paramName}}`, encodeURIComponent(paramValue));
      }
    }
  }

  // Make the API request with retry logic
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "POST", // insert is always POST
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: codeString,
    });

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`Unable to create resource; API returned ${resp.status} ${resp.statusText}: ${errorText}`) as any;
      error.status = resp.status;
      error.body = errorText;
      throw error;
    }

    return resp;
  }, {
    isRateLimitedFn: (error) => error.status === 429
  }).then((r) => r.result);

  const responseJson = await response.json();

  // Handle Google Cloud Long-Running Operations (LRO)
  // Check if this is an operation response (has kind with "operation")
  if (responseJson.kind && responseJson.kind.includes("operation")) {
    console.log(`[CREATE] LRO detected, polling for completion...`);

    // Use selfLink or construct URL from operation name
    const pollingUrl = responseJson.selfLink || `${baseUrl}${responseJson.name}`;

    // Poll the operation until it completes using new siExec.pollLRO
    const finalResource = await siExec.pollLRO({
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
          throw new Error(`Operation failed: ${JSON.stringify(body.error)}`);
        }
        
        // For create operations, get the final resource from the operation response
        // Some operations include the created resource in the response field
        if (body.response) {
          return body.response;
        }
        
        // GCP pattern: fetch the final resource from targetLink
        if (body.targetLink) {
          const resourceResponse = await fetch(body.targetLink, {
            method: "GET",
            headers: { "Authorization": `Bearer ${token}` },
          });

          if (!resourceResponse.ok) {
            throw new Error(`Failed to fetch final resource: ${resourceResponse.status}`);
          }

          return await resourceResponse.json();
        }
        
        // Fallback: return the operation body
        console.warn("[GCP] Operation completed but no response or targetLink found");
        return body;
      }
    });

    // Extract resource ID from the final resource
    const resourceId = finalResource.name || finalResource.id;

    console.log(`[CREATE] Operation complete, resourceId: ${resourceId}`);
    return {
      resourceId: resourceId ? resourceId.toString() : undefined,
      status: "ok",
      payload: finalResource,
    };
  }

  // Handle synchronous response
  const resourceId = responseJson.name || responseJson.id;

  if (resourceId) {
    return {
      resourceId: resourceId.toString(),
      status: "ok",
      payload: responseJson,
    };
  } else {
    return {
      status: "ok",
      payload: responseJson,
    };
  }
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
