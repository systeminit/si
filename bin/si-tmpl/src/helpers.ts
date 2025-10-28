/**
 * Helper utilities for error handling and other common tasks.
 *
 * @module
 */

import axios from "axios";

/**
 * Converts an unknown value to an error message string.
 *
 * This function safely extracts error messages from various error types,
 * handling both Error objects and primitive values.
 *
 * @param value - The value to convert to an error message
 * @returns A string representation of the error
 *
 * @example
 * ```ts
 * try {
 *   throw new Error("Something went wrong");
 * } catch (error) {
 *   const message = unknownValueToErrorMessage(error);
 *   console.error(message); // "Something went wrong"
 * }
 * ```
 */
export function unknownValueToErrorMessage(value: unknown): string {
  if (value instanceof Error) {
    return value.message;
  } else if (typeof value === "string") {
    return value;
  } else {
    return String(value);
  }
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
export function extractErrorDetails(error: unknown): string {
  if (axios.isAxiosError(error)) {
    const details: string[] = [
      `Status: ${error.response?.status || "unknown"}`,
      `Message: ${error.message}`,
    ];

    if (error.response?.data) {
      details.push(`Response: ${JSON.stringify(error.response.data, null, 2)}`);
    }

    if (error.config?.url) {
      details.push(
        `URL: ${error.config.method?.toUpperCase()} ${error.config.url}`,
      );
    }

    return details.join("\n");
  }

  return error instanceof Error ? error.message : String(error);
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
