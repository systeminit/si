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
  const insertApiPathJson = _.get(component.properties, ["domain", "extra", "insertApiPath"], "");
  if (!insertApiPathJson) {
    return {
      status: "error",
      message: "No insert API path metadata found - this resource may not support creation",
    };
  }

  const insertApiPath = JSON.parse(insertApiPathJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");

  // Get resourceIdStyle to determine how to extract resourceId from response
  const resourceIdStyle = _.get(component.properties, ["domain", "extra", "resourceIdStyle"], "simpleName");

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  // Build the URL
  const url = buildUrlWithParams(baseUrl, insertApiPath, component, projectId);
  const httpMethod = insertApiPath.httpMethod || "POST";

  // Make the API request with retry logic
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: httpMethod,
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: codeString,
    });

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`Unable to create resource;
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
    console.log(`[CREATE] LRO detected, polling for completion...`);

    const pollingUrl = buildLroPollingUrl(baseUrl, responseJson, component);

    // Poll until complete, then extract resourceId from the operation result
    const operationResult = await siExec.pollLRO({
      url: pollingUrl,
      headers: { "Authorization": `Bearer ${token}` },
      maxAttempts: 20,
      baseDelay: 2000,
      maxDelay: 30000,
      isCompleteFn: (_response: any, body: any) => body.status === "DONE" || body.done === true,
      isErrorFn: (_response: any, body: any) => !!body.error,
      extractResultFn: async (_response: any, body: any) => {
        if (body.error) {
          throw new Error(`Create operation failed: ${JSON.stringify(body.error)}`);
        }
        return body;
      }
    });

    // Extract resourceId from the operation result
    const resourceId = extractResourceIdFromOperation(operationResult, resourceIdStyle);
    console.log(`[CREATE] Operation complete, resourceId: ${resourceId}`);

    return {
      resourceId: resourceId ? resourceId.toString() : undefined,
      status: "ok",
    };
  }

  // Handle synchronous response
  const resourceId = extractResourceId(responseJson, resourceIdStyle === "fullPath");

  return {
    resourceId: resourceId ? resourceId.toString() : undefined,
    status: "ok",
  };
}

// ============================================================================
// Helper Functions
// ============================================================================

// Get location from component (domain only for create - no resource yet)
function getLocation(component: Input): string | undefined {
  return _.get(component.properties, ["domain", "location"]) ||
    _.get(component.properties, ["domain", "zone"]) ||
    _.get(component.properties, ["domain", "region"]);
}

// Resolve a parameter value from component properties (create uses domain only)
function resolveParamValue(
  component: Input,
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
      const supportsAutoConstruct = _.get(component.properties, ["domain", "extra", "supportsParentAutoConstruct"]) === "true";

      if (supportsAutoConstruct && location) {
        parentValue = `projects/${projectId}/locations/${location}`;
      }
    }
    return parentValue;
  }

  return _.get(component.properties, ["domain", paramName]);
}

// Build URL by replacing path parameters using RFC 6570 URI templates
// For create, parameters not in path are added as query params (e.g., GCS project)
function buildUrlWithParams(
  baseUrl: string,
  apiPath: { path: string; parameterOrder?: string[] },
  component: Input,
  projectId: string | undefined
): string {
  let url = `${baseUrl}${apiPath.path}`;
  const queryParams: string[] = [];

  if (apiPath.parameterOrder) {
    for (const paramName of apiPath.parameterOrder) {
      const paramValue = resolveParamValue(component, paramName, projectId);

      if (paramValue) {
        if (url.includes(`{+${paramName}}`)) {
          url = url.replace(`{+${paramName}}`, paramValue);
        } else if (url.includes(`{${paramName}}`)) {
          url = url.replace(`{${paramName}}`, encodeURIComponent(paramValue));
        } else {
          // Parameter not in path template - add as query parameter (e.g., GCS project)
          queryParams.push(`${paramName}=${encodeURIComponent(paramValue)}`);
        }
      }
    }
  }

  if (queryParams.length > 0) {
    url += (url.includes("?") ? "&" : "?") + queryParams.join("&");
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

// Extract resourceId from LRO operation result
function extractResourceIdFromOperation(operation: any, resourceIdStyle: string): string | undefined {
  // Check common locations for the created resource info
  // 1. response field (modern APIs)
  if (operation.response) {
    return extractResourceId(operation.response, resourceIdStyle === "fullPath");
  }

  // 2. targetLink field (Compute Engine)
  if (operation.targetLink) {
    return extractResourceIdFromUrl(operation.targetLink, resourceIdStyle === "fullPath");
  }

  // 3. metadata.target (some APIs)
  if (operation.metadata?.target) {
    return operation.metadata.target;
  }

  // 4. targetId (Compute Engine)
  if (operation.targetId) {
    return operation.targetId;
  }

  return undefined;
}

// Extract resourceId from a full URL
function extractResourceIdFromUrl(urlString: string, useFullPath: boolean): string | undefined {
  try {
    const url = new URL(urlString);
    const pathParts = url.pathname.split("/").filter(Boolean);

    if (useFullPath) {
      const projectsIdx = pathParts.indexOf("projects");
      if (projectsIdx !== -1) {
        return pathParts.slice(projectsIdx).join("/");
      }
      const versionIdx = pathParts.findIndex(p => /^v\d/.test(p));
      if (versionIdx !== -1 && versionIdx + 1 < pathParts.length) {
        return pathParts.slice(versionIdx + 1).join("/");
      }
    }

    return pathParts[pathParts.length - 1];
  } catch {
    return undefined;
  }
}

// Extract resourceId from a GCP resource response
function extractResourceId(resource: any, useFullPath: boolean): string | undefined {
  // For APIs using {+name}, extract the full path from selfLink
  if (useFullPath && resource.selfLink && typeof resource.selfLink === "string") {
    const extracted = extractResourceIdFromUrl(resource.selfLink, true);
    if (extracted) return extracted;
  }

  // For Compute Engine style APIs or fallback, use simple name/id
  return resource.name || resource.id;
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
