// Service Networking Connections update override
// This resource requires constructing name from parent + peering
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

  // Filter to only changed fields
  if (currentResource) {
    const changedFields: Record<string, any> = {};

    for (const [key, value] of Object.entries(updatePayload)) {
      if (!(key in currentResource) || !_.isEqual(value, currentResource[key])) {
        changedFields[key] = value;
      }
    }

    updatePayload = changedFields;
  }

  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");

  // Construct the connection name from parent + peering
  const parent = _.get(component.properties, ["domain", "parent"]) ||
                 _.get(component.properties, ["resource", "payload", "parent"]);
  const peering = _.get(component.properties, ["resource", "payload", "peering"]) ||
                  _.get(component.properties, ["domain", "peering"]);

  if (!parent || !peering) {
    return {
      status: "error",
      message: "Cannot update: missing parent or peering information",
    };
  }

  const connectionName = `${parent}/connections/${peering}`;

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token } = await getAccessToken(serviceAccountJson);

  // Build the URL
  const url = `${baseUrl}v1/${connectionName}`;

  // Add updateMask for the changed fields
  const updateFields = Object.keys(updatePayload);
  let finalUrl = url;
  if (updateFields.length > 0) {
    const updateMask = updateFields.join(',');
    finalUrl += `?updateMask=${encodeURIComponent(updateMask)}`;
  }

  // Make the API request with retry logic
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(finalUrl, {
      method: "PATCH",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(updatePayload),
    });

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`Unable to update resource;
Called "${finalUrl}"
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

  const responseJson = await response.json();

  // Handle Long-Running Operation if returned
  if (responseJson.name && responseJson.name.includes("operations")) {
    console.log(`[UPDATE] LRO detected, polling for completion...`);

    const pollingUrl = `${baseUrl}v1/${responseJson.name}`;

    const finalResource = await siExec.pollLRO({
      url: pollingUrl,
      headers: { "Authorization": `Bearer ${token}` },
      maxAttempts: 20,
      baseDelay: 2000,
      maxDelay: 30000,
      isCompleteFn: (response: any, body: any) => body.done === true,
      isErrorFn: (response: any, body: any) => !!body.error,
      extractResultFn: async (response: any, body: any) => {
        if (body.error) {
          throw new Error(`Operation failed: ${JSON.stringify(body.error)}`);
        }
        if (body.response) {
          return body.response;
        }
        return body;
      }
    });

    console.log(`[UPDATE] Operation complete`);
    return {
      payload: normalizeGcpResourceValues(finalResource),
      status: "ok",
    };
  }

  return {
    payload: normalizeGcpResourceValues(responseJson),
    status: "ok",
  };
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
