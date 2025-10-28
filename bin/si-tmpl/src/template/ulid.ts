/**
 * ULID (Universally Unique Lexicographically Sortable Identifier) generation utilities.
 *
 * ULIDs are 26-character alphanumeric strings that are lexicographically sortable
 * and encode timestamp information. They're used throughout System Initiative for
 * component IDs and other identifiers.
 *
 * Format: 10 characters timestamp + 16 characters randomness
 * Example: 01ARZ3NDEKTSV4RRFFQ69G5FAV
 *
 * This module wraps the Deno standard library's @std/ulid package.
 */

import { ulid } from "@std/ulid";

/**
 * Generates a ULID (Universally Unique Lexicographically Sortable Identifier).
 *
 * @returns A 26-character ULID string
 */
export function generateULID(): string {
  return ulid();
}
