// Service Networking PeeredDnsDomains refresh override
// This resource is list-only (no get API) and identified by name field
async function main(component: Input): Promise<Output> {
  const serviceAccountJson = requestStorage.getEnv("GOOGLE_APPLICATION_CREDENTIALS_JSON");
  if (!serviceAccountJson) {
    throw new Error("Google Cloud Credential not found. Please ensure a Google Cloud Credential is attached to this component.");
  }

  const { token } = await getAccessToken(serviceAccountJson);
  const baseUrl = _.get(component.properties, ["domain", "extra", "baseUrl"], "");

  // Get parent from domain or resource - parent is the full path like:
  // services/servicenetworking.googleapis.com/projects/{project}/global/networks/{network}
  const parent = _.get(component.properties, ["domain", "parent"]) ||
                 _.get(component.properties, ["resource", "payload", "parent"]);

  if (!parent) {
    return { status: "error", message: "No parent found for refresh" };
  }

  // Get the name of the peered DNS domain we're looking for
  const domainName = _.get(component.properties, ["si", "resourceId"]) ||
                     _.get(component.properties, ["domain", "name"]) ||
                     _.get(component.properties, ["resource", "payload", "name"]);

  // Build list URL
  const listUrl = `${baseUrl}v1/${parent}/peeredDnsDomains`;

  // Fetch the list of peered DNS domains
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

  // Find the array containing peered DNS domains
  let peeredDnsDomains = listData.peeredDnsDomains;
  if (!peeredDnsDomains) {
    for (const [key, value] of Object.entries(listData)) {
      if (Array.isArray(value) && key !== "unreachable" && key !== "warnings") {
        peeredDnsDomains = value;
        break;
      }
    }
  }

  // Match by name field
  for (const domain of peeredDnsDomains || []) {
    if (domainName && domain.name) {
      // Compare names - handle both short name and full path
      const domainShortName = domain.name.split("/").pop();
      const targetShortName = domainName.split("/").pop();
      if (domainShortName === targetShortName || domain.name === domainName) {
        return { payload: normalizeGcpResourceValues(domain), status: "ok" };
      }
    }
  }

  // Resource not found - may have been deleted
  console.log(`[REFRESH] PeeredDnsDomain ${domainName} not found - may have been deleted`);
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
