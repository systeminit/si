import { FetchSchemaOptions } from "../types.ts";
import * as path from "node:path";

interface DiscoveryItem {
  kind: string;
  id: string;
  name: string;
  version: string;
  title: string;
  description: string;
  discoveryRestUrl: string;
  preferred?: boolean;
}

interface DiscoveryDirectory {
  kind: "discovery#directoryList";
  discoveryVersion: string;
  items: DiscoveryItem[];
}

interface ApiVersionGroup {
  name: string;
  versions: DiscoveryItem[];
}

export async function fetchGcpDiscoveryDocuments(
  options: FetchSchemaOptions,
): Promise<void> {
  const outputDir = path.join(options.providerSchemasPath, "gcp");

  await Deno.mkdir(outputDir, { recursive: true });

  console.log("Fetching GCP API discovery directory...");
  const directoryResp = await fetchWithRetry(
    "https://www.googleapis.com/discovery/v1/apis",
  );
  if (!directoryResp.ok) {
    throw new Error(
      `Failed to fetch GCP discovery directory: ${directoryResp.statusText}`,
    );
  }

  const directory: DiscoveryDirectory = await directoryResp.json();

  const apisByName = new Map<string, ApiVersionGroup>();
  for (const item of directory.items) {
    if (!apisByName.has(item.name)) {
      apisByName.set(item.name, { name: item.name, versions: [] });
    }
    apisByName.get(item.name)!.versions.push(item);
  }

  console.log(`Found ${apisByName.size} GCP APIs, selecting best versions...`);

  const selectedApis: DiscoveryItem[] = [];
  for (const apiGroup of apisByName.values()) {
    const bestVersion = selectBestVersion(apiGroup.versions);
    selectedApis.push(bestVersion);
  }

  console.log(`Selected ${selectedApis.length} API versions to fetch`);

  let fetched = 0;
  for (const api of selectedApis) {
    const filename = `${api.name}.json`;
    const filepath = path.join(outputDir, filename);

    try {
      const docResp = await fetchWithRetry(api.discoveryRestUrl);
      if (!docResp.ok) {
        console.warn(
          `Failed to fetch ${api.name} ${api.version}: ${docResp.statusText}`,
        );
        continue;
      }

      const doc = await docResp.json();
      await Deno.writeTextFile(filepath, stableStringify(doc, 2));

      fetched++;
      if (fetched % 10 === 0) {
        console.log(`Fetched ${fetched}/${selectedApis.length} APIs...`);
      }
    } catch (e) {
      console.warn(`Error fetching ${api.name} ${api.version}: ${e}`);
    }
  }

  console.log(
    `Successfully fetched ${fetched}/${selectedApis.length} GCP discovery documents to ${outputDir}`,
  );
}

async function fetchWithRetry(
  url: string,
  retries = 3,
  delayMs = 250,
  timeoutMs = 30000,
): Promise<Response> {
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

    try {
      const response = await fetch(url, { signal: controller.signal });
      clearTimeout(timeoutId);

      if (!response.ok) {
        if (retries > 0) {
          console.warn(
            `Fetch failed for ${url} with status ${response.status}. Retrying in ${delayMs}ms...`,
          );
          await new Promise((resolve) => setTimeout(resolve, delayMs));
          return fetchWithRetry(url, retries - 1, delayMs * 2, timeoutMs);
        }
        throw new Error(
          `Fetch failed after multiple retries for ${url}: ${response.statusText}`,
        );
      }
      return response;
    } catch (error) {
      clearTimeout(timeoutId);
      throw error;
    }
  } catch (error) {
    if (retries > 0) {
      const message = error instanceof Error ? error.message : String(error);
      console.warn(
        `Fetch error for ${url}: ${message}. Retrying in ${delayMs}ms...`,
      );
      await new Promise((resolve) => setTimeout(resolve, delayMs));
      return fetchWithRetry(url, retries - 1, delayMs * 2, timeoutMs);
    }
    throw error;
  }
}

function isStableVersion(version: string): boolean {
  return !version.includes("alpha") && !version.includes("beta");
}

function selectBestVersion(versions: DiscoveryItem[]): DiscoveryItem {
  // First, try the preferred version if one exists
  const preferred = versions.find((v) => v.preferred);
  if (preferred) {
    return preferred;
  }

  // Fall back to highest stable version
  const stableVersions = versions.filter((v) => isStableVersion(v.version));
  const candidateVersions = stableVersions.length > 0
    ? stableVersions
    : versions;

  candidateVersions.sort((a, b) => b.version.localeCompare(a.version));
  return candidateVersions[0];
}

function stableStringify(obj: unknown, space = 2): string {
  function sortKeys(value: unknown): unknown {
    if (value === null || typeof value !== "object") {
      return value;
    }

    if (Array.isArray(value)) {
      return value.map(sortKeys);
    }

    const sorted: Record<string, unknown> = {};
    const keys = Object.keys(value).sort();
    for (const key of keys) {
      sorted[key] = sortKeys((value as Record<string, unknown>)[key]);
    }
    return sorted;
  }

  return JSON.stringify(sortKeys(obj), null, space);
}
