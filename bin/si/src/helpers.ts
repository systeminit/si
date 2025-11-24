import { AxiosError } from "axios";
import { Logger } from "./logger.ts";


export function unknownValueToErrorMessage(value: unknown): string {
  if (typeof value === "string") return value;

  if (value instanceof AxiosError && value.response?.data?.error?.message) {
    const status = value.response.status;
    const msg = value.response.data.error.message;
    return `HTTP ${status}: ${msg}`;
  }

  if (value instanceof Error) return value.message;

  return `Unknown Error: ${value}`;
}

export function makeStringSafeForFilename(str: string): string {
  return str.replace(/[\\/:*?"<>|]/g, "_");
}

// I kept deleting this and bringing it back when debugging API client usage, so let's keep it here
export function logAllFunctions(obj: unknown) {
  if (typeof obj !== "object" || obj === null) {
    console.log("Not an object:", obj);
    return;
  }

  const objWithIndex = obj as Record<string, unknown>;
  const allPrototypeProps = Object.getOwnPropertyNames(
    Object.getPrototypeOf(objWithIndex),
  );
  const prototypeFunctions = allPrototypeProps.filter((name) =>
    typeof objWithIndex[name] === "function"
  );
  console.log(prototypeFunctions);
}

/**
 * Extracts detailed error information from an error object, including
 * Axios-specific error details like response status and data.
 *
 * @param error - The error to extract details from
 * @returns A formatted string with error details
 *
 * @example
 * ```ts
 * try {
 *   await api.createComponent(...);
 * } catch (error) {
 *   const details = extractErrorDetails(error);
 *   console.error(details);
 *   // Status: 400
 *   // Message: Request failed with status code 400
 *   // Response: {"error": "Invalid component ID"}
 *   // URL: POST https://api.systeminit.com/v1/component
 * }
 * ```
 */
export function extractErrorDetails(
  error: unknown,
  includeStack = false,
): string {
  if (error instanceof AxiosError) {
    const details: string[] = [
      `Status: ${error.response?.status || "unknown"}`,
      `Message: ${error.message}`,
    ];

    if (error.config?.url) {
      details.push(
        `URL: ${error.config.method?.toUpperCase()} ${error.config.url}`,
      );
    }

    if (error.config?.data) {
      details.push(
        `Request Body: ${JSON.stringify(error.config.data, null, 2)}`,
      );
    }

    if (error.response?.data) {
      details.push(`Response: ${JSON.stringify(error.response.data, null, 2)}`);
    }

    if (includeStack && error.stack) {
      details.push(`Stack: ${error.stack}`);
    }

    return details.join("\n");
  }

  if (error instanceof Error) {
    if (includeStack && error.stack) {
      return `${error.message}\n${error.stack}`;
    }
    return error.message;
  }

  return String(error);
}

/**
 * Logs component loading information with schema name lookup.
 *
 * This helper consolidates the common pattern of fetching a schema name
 * and logging component information with progress tracking.
 *
 * @param ctx - The template context (must have logger and getSchemaName method)
 * @param component - Component object with schemaId, name, and optional attributes
 * @param workspaceId - The workspace ID
 * @param changeSetId - The change set ID
 * @param messageTemplate - Log message template (e.g., "Loaded baseline component {schemaName} {siName} ({current}/{total})")
 * @param current - Current progress count (optional)
 * @param total - Total count (optional)
 *
 * @example
 * ```ts
 * await logComponentWithSchema(
 *   ctx,
 *   component,
 *   workspaceId,
 *   changeSetId,
 *   "Loaded baseline component {schemaName} {siName} ({current}/{total})",
 *   5,
 *   10
 * );
 * ```
 */
export async function logComponentWithSchema(
  ctx: {
    logger: {
      info: (message: string, context: Record<string, unknown>) => void;
    };
    getSchemaName: (
      workspaceId: string,
      changeSetId: string,
      schemaId: string,
    ) => Promise<string>;
  },
  component: {
    schemaId: string;
    name: string;
    attributes?: Record<string, unknown>;
  },
  workspaceId: string,
  changeSetId: string,
  messageTemplate: string,
  current?: number,
  total?: number,
): Promise<void> {
  const schemaName = await ctx.getSchemaName(
    workspaceId,
    changeSetId,
    component.schemaId,
  );
  const siName = component.attributes?.["si/name"] || component.name;

  const logContext: Record<string, unknown> = {
    schemaName,
    siName,
  };

  if (current !== undefined) {
    logContext.current = current;
  }

  if (total !== undefined) {
    logContext.total = total;
  }

  ctx.logger.info(messageTemplate, logContext);
}

/**
 * Extracts the category from a search pattern like "Fastly::*"
 * Returns the category or null if pattern is invalid
 */
export function extractCategoryFromPattern(pattern: string): string | null {
  // Match patterns like "Category::*" or "Category::Subcategory::*"
  const match = pattern.match(/^(.+)::\*$/);
  if (match && match[1]) {
    return match[1];
  }
  // If pattern is just "*", return null to search all
  if (pattern === "*") {
    return null;
  }
  return null;
}

/**
 * Detects if a schema name is a search pattern (e.g., "Fastly::*")
 */
export function isSearchPattern(schemaName: string): boolean {
  return schemaName.includes("*");
}

/**
 * Searches for schemas using the search API
 */
export async function searchSchemas(
  api: SchemasApi,
  logger: Logger,
  changeSetCoord: ChangeSetCoordinate,
  pattern: string,
): Promise<string[]> {
  const category = extractCategoryFromPattern(pattern);

  logger.info(`Searching for schemas matching pattern: ${pattern}`);
  if (category) {
    logger.info(`  Using category filter: ${category}`);
  } else {
    logger.info(`  Searching all schemas`);
  }

  const response = await api.searchSchemas({
    workspaceId: changeSetCoord.workspaceId,
    changeSetId: changeSetCoord.changeSetId,
    searchSchemasV1Request: {
      category: category,
    },
  });

  const schemaNames = response.data.schemas.map((s) => s.schemaName);
  logger.info(`  Found ${schemaNames.length} matching schema(s)`);

  return schemaNames;
}
