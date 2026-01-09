// Service Networking Connections delete override
// This resource requires:
// - Constructing name from parent + peering
// - POST method with consumerNetwork in request body
async function main(component: Input): Promise<Output> {
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token } = await getAccessToken(serviceAccountJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");

  // Construct the connection name from parent + peering
  const parent = _.get(component.properties, ["domain", "parent"]) ||
                 _.get(component.properties, ["resource", "payload", "parent"]);
  const peering = _.get(component.properties, ["resource", "payload", "peering"]) ||
                  _.get(component.properties, ["domain", "peering"]);
  const network = _.get(component.properties, ["domain", "network"]) ||
                  _.get(component.properties, ["resource", "payload", "network"]);

  if (!parent || !peering) {
    return {
      status: "error",
      message: "Cannot delete: missing parent or peering information",
    };
  }

  const connectionName = `${parent}/connections/${peering}`;
  const url = `${baseUrl}v1/${connectionName}`;

  // Service Networking delete requires consumerNetwork in request body
  const requestBody = network ? JSON.stringify({ consumerNetwork: network }) : undefined;

  // Make the API request with retry logic
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "POST", // Service Networking uses POST for deleteConnection
      headers: {
        "Authorization": `Bearer ${token}`,
        ...(requestBody ? { "Content-Type": "application/json" } : {}),
      },
      body: requestBody,
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
    return { status: "ok" };
  }

  // Handle 204 No Content
  if (response.status === 204) {
    return { status: "ok" };
  }

  // Try to parse response body
  const responseText = await response.text();
  if (!responseText) {
    return { status: "ok" };
  }

  const responseJson = JSON.parse(responseText);

  // Handle Long-Running Operation if returned
  if (responseJson.name && responseJson.name.includes("operations")) {
    console.log(`[DELETE] LRO detected, polling for completion...`);

    const pollingUrl = `${baseUrl}v1/${responseJson.name}`;

    await siExec.pollLRO({
      url: pollingUrl,
      headers: { "Authorization": `Bearer ${token}` },
      maxAttempts: 20,
      baseDelay: 2000,
      maxDelay: 30000,
      isCompleteFn: (response, body) => body.done === true,
      isErrorFn: (response, body) => !!body.error,
      extractResultFn: async (response, body) => {
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
