async function main(component: Input): Promise<Output> {
  // Get the resource to ensure the instance group exists
  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: "error",
      message: "Instance group must be created before removing instances",
    };
  }

  // Get instances to remove from domain
  const instancesToRemove = _.get(component.properties, ["domain", "instances"], []);

  if (!instancesToRemove || instancesToRemove.length === 0) {
    return {
      status: "error",
      message: "No instances specified to remove",
    };
  }

  // Get zone and instance group name
  const zone = _.get(component.properties, ["domain", "zone"]);
  const instanceGroupName = resource.name || _.get(component.properties, ["domain", "name"]);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "https://compute.googleapis.com/compute/v1/");

  if (!zone || !instanceGroupName) {
    return {
      status: "error",
      payload: resource,
      message: "Missing required zone or instance group name",
    };
  }

  // Get authentication token
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token, projectId } = await getAccessToken(serviceAccountJson);

  if (!projectId) {
    return {
      status: "error",
      payload: resource,
      message: "Could not determine project ID from service account",
    };
  }

  // Build the removeInstances API URL
  const url = `${baseUrl}projects/${projectId}/zones/${zone}/instanceGroups/${instanceGroupName}/removeInstances`;

  // Format instances for the API request
  // Each instance should be a selfLink URL or we construct it
  const instanceReferences = instancesToRemove.map((inst: string | { instance: string }) => {
    const instanceUrl = typeof inst === "string" ? inst : inst.instance;
    // If it's already a full URL, use it; otherwise construct it
    if (instanceUrl.startsWith("http")) {
      return { instance: instanceUrl };
    }
    // Assume it's a name or partial path, construct full URL
    // Format: projects/{project}/zones/{zone}/instances/{instance}
    if (instanceUrl.startsWith("projects/")) {
      return { instance: `${baseUrl}${instanceUrl}` };
    }
    // Just an instance name, construct full path using same zone
    return { instance: `${baseUrl}projects/${projectId}/zones/${zone}/instances/${instanceUrl}` };
  });

  const requestBody = {
    instances: instanceReferences,
  };

  console.log(`[REMOVE INSTANCES] Removing ${instanceReferences.length} instance(s) from instance group ${instanceGroupName}`);

  // Make the API request
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(url, {
      method: "POST",
      headers: {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(requestBody),
    });

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`Unable to remove instances; API returned ${resp.status} ${resp.statusText}: ${errorText}`) as any;
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
    console.log(`[REMOVE INSTANCES] LRO detected, polling for completion...`);

    const pollingUrl = responseJson.selfLink || `${baseUrl}${responseJson.name}`;

    await siExec.pollLRO({
      url: pollingUrl,
      headers: { "Authorization": `Bearer ${token}` },
      maxAttempts: 20,
      baseDelay: 2000,
      maxDelay: 30000,
      isCompleteFn: (_response, body) => body.status === "DONE",
      isErrorFn: (_response, body) => !!body.error,
      extractResultFn: async (_response, body) => {
        if (body.error) {
          throw new Error(`Operation failed: ${JSON.stringify(body.error)}`);
        }
        return body;
      }
    });

    console.log(`[REMOVE INSTANCES] Successfully removed ${instanceReferences.length} instance(s)`);
  }

  return {
    status: "ok",
    payload: resource,
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
