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

  // Make the API request
  const response = await fetch(url, {
    method: "POST", // insert is always POST
    headers: {
      "Authorization": `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: codeString,
  });

  if (!response.ok) {
    const errorText = await response.text();
    return {
      status: "error",
      message: `Unable to create resource; API returned ${response.status} ${response.statusText}: ${errorText}`,
    };
  }

  const responseJson = await response.json();

  // Handle Google Cloud Long-Running Operations (LRO)
  // Check if this is an operation response (has kind with "operation")
  if (responseJson.kind && responseJson.kind.includes("operation")) {
    console.log(`[CREATE] LRO detected, polling for completion...`);

    // Poll the operation until it completes
    const finalResource = await pollOperation(responseJson, baseUrl, token);

    // Extract resource ID from the final resource
    const resourceId = finalResource.name || finalResource.id;

    console.log(`[CREATE] Operation complete, resourceId: ${resourceId}`);
    return {
      resourceId: resourceId ? resourceId.toString() : undefined,
      status: "ok",
      payload: normalizeGcpResourceValues(finalResource),
    };
  }

  // Handle synchronous response
  const resourceId = responseJson.name || responseJson.id;

  if (resourceId) {
    return {
      resourceId: resourceId.toString(),
      status: "ok",
      payload: normalizeGcpResourceValues(responseJson),
    };
  } else {
    return {
      status: "ok",
      payload: normalizeGcpResourceValues(responseJson),
    };
  }
}

async function pollOperation(
  operation: any,
  baseUrl: string,
  token: string,
): Promise<any> {
  const delay = (ms: number) => new Promise(res => setTimeout(res, ms));

  // Use selfLink or construct URL from operation name
  const pollingUrl = operation.selfLink || `${baseUrl}${operation.name}`;

  console.log(`[LRO] Polling URL: ${pollingUrl}`);
  console.log(`[LRO] Initial operation:`, JSON.stringify(operation, null, 2));

  // Poll until operation status is DONE
  let currentOp = operation;
  while (currentOp.status !== "DONE") {
    await delay(2000); // Simple 2-second polling interval

    console.log(`[LRO] Polling operation status...`);
    const response = await fetch(pollingUrl, {
      method: "GET",
      headers: { "Authorization": `Bearer ${token}` },
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error(`[LRO] Polling failed: ${response.status} ${response.statusText}`);
      console.error(`[LRO] Error body: ${errorText}`);
      throw new Error(`Operation polling failed: ${response.status} ${response.statusText} - ${errorText}`);
    }

    currentOp = await response.json();
    console.log(`[LRO] Operation status: ${currentOp.status}`);
  }

  console.log(`[LRO] Operation completed with status: ${currentOp.status}`);

  // Check for operation error
  if (currentOp.error) {
    throw new Error(`Operation failed: ${JSON.stringify(currentOp.error)}`);
  }

  // Fetch the final resource from targetLink
  if (!currentOp.targetLink) {
    throw new Error("Operation completed but no targetLink found");
  }

  const resourceResponse = await fetch(currentOp.targetLink, {
    method: "GET",
    headers: { "Authorization": `Bearer ${token}` },
  });

  if (!resourceResponse.ok) {
    throw new Error(`Failed to fetch resource: ${resourceResponse.status}`);
  }

  return await resourceResponse.json();
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
