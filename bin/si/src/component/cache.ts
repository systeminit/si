import { extname } from "@std/path";
import { stringify as stringifyYaml } from "@std/yaml";
import type { Logger } from "@logtape/logtape";

/**
 * Recursively removes undefined values from an object for YAML serialization.
 * YAML stringify cannot handle undefined values.
 *
 * @param obj - The object to clean
 * @returns Cleaned object with undefined values removed
 */
export function cleanForYaml(obj: unknown): unknown {
  if (obj === null || obj === undefined) {
    return null;
  }
  if (Array.isArray(obj)) {
    return obj.map(cleanForYaml);
  }
  if (typeof obj === "object") {
    const cleaned: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj as Record<string, unknown>)) {
      if (value !== undefined) {
        cleaned[key] = cleanForYaml(value);
      }
    }
    return cleaned;
  }
  return obj;
}

/**
 * Container for cached component data.
 *
 * Used to persist component information to disk for debugging,
 * inspection, or offline analysis.
 *
 * The cache can be written in JSON or YAML format based on file extension.
 *
 * @example
 * ```ts
 * // Cache structure when written to file
 * {
 *   "componentId": "01HQXYZ...",
 *   "schemaId": "01HQABC...",
 *   "schemaName": "AWS EC2 Instance",
 *   "resourceId": "i-1234567890abcdef0",
 *   "qualified": true,
 *   "attributes": {
 *     "si": { "name": "my-instance" },
 *     "domain": { "instanceType": "t3.micro" },
 *     "secrets": {},
 *     "resource_value": { "InstanceId": "i-1234567890abcdef0" }
 *   },
 *   "resourceData": { "InstanceId": "i-1234567890abcdef0" },
 *   "qualifications": [
 *     { "name": "ValidConfig", "status": "success", "message": "Valid" }
 *   ],
 *   "actions": [
 *     { "id": "action-123", "name": "create", "state": "queued" }
 *   ]
 * }
 * ```
 */
export interface ComponentGetCache {
  /** The component's unique identifier */
  componentId: string;
  /** The schema ID for this component */
  schemaId: string;
  /** Human-readable schema name */
  schemaName: string;
  /** Resource ID if the component is backed by a real resource */
  resourceId?: string;
  /** Whether the component is marked for deletion */
  toDelete: boolean;
  /** Whether the component can be upgraded to a newer schema variant */
  canBeUpgraded: boolean;
  /** Whether the component passes all qualifications */
  qualified: boolean;
  /**
   * Filtered attributes with full paths as keys
   * (only includes /si, /domain, /secrets, /resource_value paths)
   */
  attributes: Record<string, unknown>;
  /** Raw resource data if available */
  resourceData?: Record<string, unknown>;
  /** Resource payload data from /resource/payload if available */
  resource?: Record<string, unknown>;
  /** List of qualification results */
  qualifications: Array<{
    /** Name of the qualification */
    name: string;
    /** Status (success, failure, warning, etc.) */
    status: string;
    /** Optional log message from qualification */
    message?: string;
  }>;
  /** List of enqueued actions for this component */
  actions: Array<{
    /** Action ID */
    id: string;
    /** Action name (create, update, delete, etc.) */
    name: string;
    /** Action state (queued, running, success, failure) */
    state: string;
  }>;
}

/**
 * Cache component data to a file.
 * Format (JSON/YAML) is determined by file extension.
 *
 * @param cacheData - The component data to cache
 * @param filePath - Path where cache file should be written
 * @param logger - Logger instance for output
 */
export async function cacheComponentData(
  cacheData: ComponentGetCache,
  filePath: string,
  logger: Logger,
): Promise<void> {
  logger.info(`Caching component data to {filePath}`, { filePath });

  // Determine format from extension
  const ext = extname(filePath).toLowerCase();
  let content: string;

  if (ext === ".json") {
    content = JSON.stringify(cacheData, null, 2);
  } else if (ext === ".yaml" || ext === ".yml") {
    // Clean undefined values as YAML stringify cannot handle them
    content = stringifyYaml(cleanForYaml(cacheData));
  } else {
    throw new Error(
      `Unsupported cache file format: ${ext}. Use .json, .yaml, or .yml`,
    );
  }

  await Deno.writeTextFile(filePath, content);

  logger.info(`Cached component data to {filePath}`, { filePath });
}
