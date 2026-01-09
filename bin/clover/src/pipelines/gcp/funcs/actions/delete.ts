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

  // Build the URL
  const url = buildUrlWithParams(baseUrl, deleteApiPath, component, projectId, { resourceId });

  // Make the API request with retry logic
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: httpMethod,
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
    isRateLimitedFn: (error: any) => error.status === 429
  }).then((r: any) => r.result);

  // Handle 404 as success for delete operations
  if (response.status === 404) {
    return { status: "ok" };
  }

  // Handle 204 No Content (common for successful deletes like GCS)
  if (response.status === 204) {
    return { status: "ok" };
  }

  // Try to parse response body - some APIs return empty body on success
  const responseText = await response.text();
  if (!responseText) {
    return { status: "ok" };
  }

  const responseJson = JSON.parse(responseText);

  // Check if this resource uses Long-Running Operations based on metadata
  const lroStyle = _.get(component.properties, ["domain", "extra", "lroStyle"], "none");

  // Detect LRO response - only check if lroStyle indicates LRO support
  const isLRO = lroStyle !== "none" && (
    (responseJson.kind && responseJson.kind.includes("operation")) ||
    responseJson.operationType ||
    (responseJson.name && responseJson.name.startsWith("operations/"))
  );

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
      isCompleteFn: (_response: any, body: any) => body.status === "DONE" || body.done === true,
      isErrorFn: (_response: any, body: any) => !!body.error,
      extractResultFn: async (_response: any, body: any) => {
        if (body.error) {
          throw new Error(`Delete operation failed: ${JSON.stringify(body.error)}`);
        }
        return body;
      }
    });

    console.log(`[DELETE] Operation complete`);
  }

  return { status: "ok" };
}

// ============================================================================
// Helper Functions
// ============================================================================

// Get location from component, checking resource payload first then domain
function getLocation(component: Input): string | undefined {
  return _.get(component.properties, ["resource", "payload", "location"]) ||
    _.get(component.properties, ["domain", "location"]) ||
    _.get(component.properties, ["domain", "zone"]) ||
    _.get(component.properties, ["domain", "region"]);
}

// Resolve a parameter value from component properties
function resolveParamValue(
  component: Input,
  paramName: string,
  projectId: string | undefined
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

      if (supportsAutoConstruct && location) {
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

// Check if resourceId is already a full path matching the API path structure
// Uses proper segment matching (not substring) to avoid false positives
function isFullResourcePath(resourceId: string, pathTemplate: string): boolean {
  if (!resourceId.includes('/')) return false;

  const templateSegments = pathTemplate.split('/').filter(s => !s.startsWith('{'));
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
  options: { resourceId?: string } = {}
): string {
  // If resourceId is already a full path matching the API structure, use it directly
  if (options.resourceId && isFullResourcePath(options.resourceId, apiPath.path)) {
    return `${baseUrl}${options.resourceId}`;
  }

  let url = `${baseUrl}${apiPath.path}`;

  if (apiPath.parameterOrder) {
    const lastParam = apiPath.parameterOrder[apiPath.parameterOrder.length - 1];

    for (const paramName of apiPath.parameterOrder) {
      let paramValue: string | undefined;

      // For the resource identifier, use resourceId
      if (options.resourceId && paramName === lastParam) {
        paramValue = options.resourceId;
      } else {
        paramValue = resolveParamValue(component, paramName, projectId);
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
