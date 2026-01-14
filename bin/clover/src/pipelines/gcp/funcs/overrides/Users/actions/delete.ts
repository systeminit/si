// SQL Admin Users: delete requires 'name' as query parameter
// The generic delete doesn't add it since it's not in parameterOrder

async function main(component: Input): Promise<Output> {
  const deleteApiPathJson = _.get(component.properties, ["domain", "extra", "deleteApiPath"], "");
  if (!deleteApiPathJson) {
    return { status: "error", message: "No delete API path metadata found" };
  }

  const deleteApiPath = JSON.parse(deleteApiPathJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");

  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  // Build URL with path parameters
  const project = projectId;
  const instance = _.get(component.properties, ["domain", "instance"]) ||
                   _.get(component.properties, ["resource", "payload", "instance"]);

  if (!project || !instance) {
    return { status: "error", message: "Missing project or instance" };
  }

  let url = `${baseUrl}${deleteApiPath.path}`
    .replace("{project}", encodeURIComponent(project))
    .replace("{instance}", encodeURIComponent(instance));

  // Add required 'name' query parameter for SQL Admin Users
  const userName = _.get(component.properties, ["domain", "name"]) ||
                   _.get(component.properties, ["resource", "payload", "name"]);
  if (!userName) {
    return { status: "error", message: "User name is required for deletion" };
  }
  url += `?name=${encodeURIComponent(userName)}`;

  // Add optional 'host' query parameter if present (for MySQL users)
  const host = _.get(component.properties, ["domain", "host"]) ||
               _.get(component.properties, ["resource", "payload", "host"]);
  if (host) {
    url += `&host=${encodeURIComponent(host)}`;
  }

  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "DELETE",
      headers: { "Authorization": `Bearer ${token}` },
    });

    if (resp.status === 404) {
      return resp; // Already deleted
    }

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`Delete failed: ${resp.status} ${resp.statusText}: ${errorText}`) as any;
      error.status = resp.status;
      throw error;
    }

    return resp;
  }, {
    isRateLimitedFn: (error) => error.status === 429
  }).then((r) => r.result);

  if (response.status === 404 || response.status === 204) {
    return { status: "ok" };
  }

  // Handle LRO response if any
  const responseText = await response.text();
  if (responseText) {
    const responseJson = JSON.parse(responseText);
    if (responseJson.kind && responseJson.kind.includes("operation")) {
      console.log(`[DELETE] LRO detected, polling for completion...`);
      const pollingUrl = responseJson.selfLink || `${baseUrl}${responseJson.name}`;

      await siExec.pollLRO({
        url: pollingUrl,
        headers: { "Authorization": `Bearer ${token}` },
        maxAttempts: 20,
        baseDelay: 2000,
        maxDelay: 30000,
        isCompleteFn: (response, body) => body.status === "DONE",
        isErrorFn: (response, body) => !!body.error,
        extractResultFn: async (response, body) => {
          if (body.error) {
            throw new Error(`Delete operation failed: ${JSON.stringify(body.error)}`);
          }
          return body;
        }
      });
    }
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
