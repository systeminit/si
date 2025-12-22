/**
 * Shared utilities for workspace operations
 *
 * @module
 */

/**
 * Type guard for API errors with response status
 */
export function hasResponseStatus(
  error: unknown,
): error is { response: { status: number } } {
  if (!error || typeof error !== "object") return false;
  const err = error as { response?: unknown };
  if (!err.response || typeof err.response !== "object") return false;
  const response = err.response as { status?: unknown };
  return typeof response.status === "number";
}
