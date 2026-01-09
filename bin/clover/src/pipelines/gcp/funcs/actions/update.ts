async function main(component: Input): Promise<Output> {
  // Get the generated code from code gen function
  const codeString = component.properties.code?.["Google Cloud Update Code Gen"]?.code;
  if (!codeString) {
    return {
      status: "error",
      message: "Could not find Google Cloud Update Code Gen code for resource",
    };
  }

  let updatePayload = JSON.parse(codeString);

  // Get current resource state to compare
  const currentResource = component.properties?.resource?.payload;

  // Filter to only changed fields (GCP PATCH requires this for some resources)
  if (currentResource) {
    const changedFields: Record<string, any> = {};

    for (const [key, value] of Object.entries(updatePayload)) {
      // Include field if it doesn't exist in current resource or has different value
      if (!(key in currentResource) || !_.isEqual(value, currentResource[key])) {
        changedFields[key] = value;
      }
    }

    updatePayload = changedFields;

    // GCP requires fingerprint for updates to prevent concurrent modifications
    if (currentResource.fingerprint) {
      updatePayload.fingerprint = currentResource.fingerprint;
    }
  }

  // Try to get update API path first, fall back to patch
  let updateApiPathJson = _.get(component.properties, ["domain", "extra", "updateApiPath"], "");
  if (!updateApiPathJson) {
    updateApiPathJson = _.get(component.properties, ["domain", "extra", "patchApiPath"], "");
  }

  if (!updateApiPathJson) {
    return {
      status: "error",
      message: "No update or patch API path metadata found - this resource may not support updates",
    };
  }

  const updateApiPath = JSON.parse(updateApiPathJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");

  // Get resourceId
  const resourceId = component.properties?.si?.resourceId;
  if (!resourceId) {
    return {
      status: "error",
      message: "No resource ID found for update",
    };
  }

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  // Build the URL
  let url = buildUrlWithParams(baseUrl, updateApiPath, component, projectId, { resourceId });

  // Add updateMask query parameter (GCP APIs require/benefit from specifying which fields are being updated)
  const updateFields = Object.keys(updatePayload).filter(k => k !== 'fingerprint');
  if (updateFields.length > 0) {
    const updateMask = updateFields.join(',');
    url += (url.includes('?') ? '&' : '?') + `updateMask=${encodeURIComponent(updateMask)}`;
  }

  // Make the API request with retry logic
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: updateApiPath.httpMethod,
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(updatePayload),
    });

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`Unable to update resource;
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

  const responseJson = await response.json();

  // Check if this resource uses Long-Running Operations based on metadata
  const lroStyle = _.get(component.properties, ["domain", "extra", "lroStyle"], "none");

  // Detect LRO response - only check if lroStyle indicates LRO support
  const isLRO = lroStyle !== "none" && (
    (responseJson.kind && responseJson.kind.includes("operation")) ||
    responseJson.operationType ||
    (responseJson.name && responseJson.name.startsWith("operations/"))
  );

  if (isLRO) {
    console.log(`[UPDATE] LRO detected, polling for completion...`);

    const pollingUrl = buildLroPollingUrl(baseUrl, responseJson, component);

    // Poll until complete - don't fetch final resource, let refresh handle that
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
          throw new Error(`Update operation failed: ${JSON.stringify(body.error)}`);
        }
        return body;
      }
    });

    console.log(`[UPDATE] Operation complete`);
  }

  // Return success - refresh will fetch the final resource state
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
  if (options.resourceId && isFullResourcePath(options.resourceId, apiPath.path)) {
    return `${baseUrl}${options.resourceId}`;
  }

  let url = `${baseUrl}${apiPath.path}`;

  if (apiPath.parameterOrder) {
    const lastParam = apiPath.parameterOrder[apiPath.parameterOrder.length - 1];

    for (const paramName of apiPath.parameterOrder) {
      let paramValue: string | undefined;

      if (options.resourceId && paramName === lastParam) {
        paramValue = options.resourceId;
      } else {
        paramValue = resolveParamValue(component, paramName, projectId);
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

  return url;
}

// Build LRO polling URL from operation response
function buildLroPollingUrl(baseUrl: string, responseJson: any, component: Input): string {
  if (responseJson.selfLink) {
    return responseJson.selfLink;
  }

  // Extract API version from paths to construct polling URL
  const insertApiPathJson = _.get(component.properties, ["domain", "extra", "insertApiPath"], "");
  const getApiPathJson = _.get(component.properties, ["domain", "extra", "getApiPath"], "");
  const pathJson = insertApiPathJson || getApiPathJson;

  let apiVersion = "";
  if (pathJson) {
    const apiPath = JSON.parse(pathJson);
    const versionMatch = apiPath.path?.match(/^(v\d+)\//);
    if (versionMatch) {
      apiVersion = versionMatch[1] + "/";
    }
  }

  return `${baseUrl}${apiVersion}${responseJson.name}`;
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
