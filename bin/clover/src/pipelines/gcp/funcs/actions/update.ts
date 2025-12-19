async function main(component: Input): Promise<Output> {
  // Get the generated code from code gen function
  const codeString = component.properties.code?.["Google Cloud Update Code Gen"]?.code;
  if (!codeString) {
    return {
      status: "error",
      message: "Could not find Google Cloud Update Code Gen code for resource",
    };
  }

  // Parse the generated payload
  const fullPayload = JSON.parse(codeString);

  // Get PropUsageMap to filter out createOnly properties
  const propUsageMapJson = _.get(
    component.properties,
    ["domain", "extra", "PropUsageMap"],
    "",
  );

  let updatePayload = fullPayload;

  // Filter out createOnly properties
  if (propUsageMapJson) {
    try {
      const propUsageMap = JSON.parse(propUsageMapJson);
      if (
        Array.isArray(propUsageMap.createOnly) &&
        Array.isArray(propUsageMap.updatable)
      ) {
        // Remove createOnly properties from the payload
        for (const createOnlyPath of propUsageMap.createOnly) {
          // Convert /domain/propertyName to propertyName
          const propName = createOnlyPath.replace(/^\/domain\//, "");
          delete updatePayload[propName];
        }
      }
    } catch (e) {
      console.log(`Warning: Failed to parse PropUsageMap: ${e}`);
    }
  }

  // Get current resource state to compare
  const currentResource = component.properties?.resource?.payload;

  // Filter to only changed fields (GCP PATCH requires this for some resources)
  // Only compare fields that exist in the current resource - if a field doesn't exist
  // in the resource response, we shouldn't try to update it
  if (currentResource) {
    const changedFields: Record<string, any> = {};

    for (const [key, value] of Object.entries(updatePayload)) {
      // Only include field if it exists in current resource AND is different
      if (key in currentResource && !_.isEqual(value, currentResource[key])) {
        changedFields[key] = value;
      }
    }

    updatePayload = changedFields;

    // GCP requires fingerprint for updates to prevent concurrent modifications
    // Always include the fingerprint from the current resource if it exists
    if (currentResource.fingerprint) {
      updatePayload.fingerprint = currentResource.fingerprint;
    }
  }

  // Try to get update API path first, fall back to patch
  let updateApiPathJson = _.get(
    component.properties,
    ["domain", "extra", "updateApiPath"],
    "",
  );

  if (!updateApiPathJson) {
    updateApiPathJson = _.get(
      component.properties,
      ["domain", "extra", "patchApiPath"],
      "",
    );
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

  // Build the URL by replacing path parameters
  let url = `${baseUrl}${updateApiPath.path}`;

  // Replace path parameters with values from resource_value or domain
  if (updateApiPath.parameterOrder) {
    for (const paramName of updateApiPath.parameterOrder) {
      let paramValue;

      // For the resource identifier, use resourceId
      if (paramName === updateApiPath.parameterOrder[updateApiPath.parameterOrder.length - 1]) {
        paramValue = resourceId;
      } else if (paramName === "project") {
        // Use extracted project_id for project parameter
        paramValue = projectId;
      } else {
        paramValue = _.get(component.properties, ["resource", "payload", paramName]) ||
                     _.get(component.properties, ["domain", paramName]);

        // GCP often returns full URLs for reference fields (e.g., region, zone, network)
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
      const error = new Error(`Unable to update resource; API returned ${resp.status} ${resp.statusText}: ${errorText}`) as any;
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
  if (responseJson.kind && responseJson.kind.includes("operation")) {
    console.log(`[UPDATE] LRO detected, polling for completion...`);

    // Use selfLink or construct URL from operation name
    const pollingUrl = responseJson.selfLink || `${baseUrl}${responseJson.name}`;

    // Poll the operation until it completes using new siExec.pollLRO
    const finalResource = await siExec.pollLRO({
      url: pollingUrl,
      headers: { "Authorization": `Bearer ${token}` },
      maxAttempts: 20,
      baseDelay: 2000,
      maxDelay: 30000,
      isCompleteFn: (response: any, body: any) => body.status === "DONE",
      isErrorFn: (response: any, body: any) => !!body.error,
      extractResultFn: async (response: any, body: any) => {
        // If operation has error, throw it
        if (body.error) {
          throw new Error(`Operation failed: ${JSON.stringify(body.error)}`);
        }
        
        // For update operations, fetch the final resource from targetLink
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

        // Some operations include the updated resource in the response field
        if (body.response) {
          return body.response;
        }
        
        // Fallback: return the operation body
        console.warn("[GCP] Operation completed but no response or targetLink found");
        return body;
      }
    });

    console.log(`[UPDATE] Operation complete`);
    return {
      payload: normalizeGcpResourceValues(finalResource),
      status: "ok",
    };
  }

  // Handle synchronous response
  return {
    payload: normalizeGcpResourceValues(responseJson),
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

// URL normalization for GCP resource values
const GCP_URL_PATTERN = /^https:\/\/[^/]*\.?googleapis\.com\//;
const LOCATION_SEGMENTS = new Set(["regions", "zones", "locations"]);

function normalizeGcpResourceValues<T>(obj: T): T {
  if (obj === null || obj === undefined) return obj;
  if (Array.isArray(obj)) return obj.map(item => normalizeGcpResourceValues(item)) as T;
  if (typeof obj === "object") {
    const normalized: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj)) {
      if (typeof value === "string" && GCP_URL_PATTERN.test(value)) {
        const pathParts = new URL(value).pathname.split("/").filter(Boolean);
        if (pathParts.length >= 2 && LOCATION_SEGMENTS.has(pathParts[pathParts.length - 2])) {
          normalized[key] = pathParts[pathParts.length - 1];
        } else {
          const projectsIdx = pathParts.indexOf("projects");
          if (projectsIdx !== -1) {
            normalized[key] = pathParts.slice(projectsIdx).join("/");
          } else {
            // For non-project APIs (e.g., Storage), extract after API version (v1, v2, etc.)
            const versionIdx = pathParts.findIndex(p => /^v\d+/.test(p));
            normalized[key] = versionIdx !== -1 && versionIdx + 1 < pathParts.length
              ? pathParts.slice(versionIdx + 1).join("/")
              : pathParts[pathParts.length - 1] || value;
          }
        }
      } else if (typeof value === "object" && value !== null) {
        normalized[key] = normalizeGcpResourceValues(value);
      } else {
        normalized[key] = value;
      }
    }
    return normalized as T;
  }
  return obj;
}
