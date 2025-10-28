/**
 * Helper utilities for error handling and other common tasks.
 *
 * @module
 */

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
