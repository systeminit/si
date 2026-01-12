// Service Networking Connections refresh override
// This resource is list-only (no get API) and identified by network field, not name/id
async function main(component: Input): Promise<Output> {
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token } = await getAccessToken(serviceAccountJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");
  const parent = _.get(component.properties, ["domain", "parent"]);
  const domainNetwork = _.get(component.properties, ["domain", "network"]);

  if (!parent) {
    return { status: "error", message: "No parent found for refresh" };
  }

  // Build list URL - Service Networking requires network as query parameter
  let listUrl = `${baseUrl}v1/${parent}/connections`;
  if (domainNetwork) {
    listUrl += `?network=${encodeURIComponent(domainNetwork)}`;
  }

  // Fetch connections list
  const response = await siExec.withRetry(async () => {
    const resp = await fetch(listUrl, {
      method: "GET",
      headers: { "Authorization": `Bearer ${token}` },
    });

    if (!resp.ok) {
      const errorText = await resp.text();
      const error = new Error(`Unable to refresh resource;
Called "${listUrl}"
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

  const listData = await response.json();

  // Find the array containing connections
  let connections = listData.connections;
  if (!connections) {
    for (const [key, value] of Object.entries(listData)) {
      if (Array.isArray(value) && key !== "unreachable" && key !== "warnings") {
        connections = value;
        break;
      }
    }
  }

  // Match by network field (Service Networking Connections don't have standard name/id)
  for (const connection of connections || []) {
    if (domainNetwork && connection.network) {
      // Normalize both for comparison (handle project number vs project name differences)
      const domainNetworkName = domainNetwork.split("/").pop();
      const connectionNetworkName = connection.network.split("/").pop();
      if (domainNetworkName && connectionNetworkName && domainNetworkName === connectionNetworkName) {
        return { payload: normalizeGcpResourceValues(connection), status: "ok" };
      }
    }
  }

  // Resource not found - may have been deleted
  console.log(`[REFRESH] Service Networking Connection for network ${domainNetwork} not found - may have been deleted`);
  return { status: "ok", payload: null };
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
