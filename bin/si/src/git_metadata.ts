// This file contains placeholder values for local development.
// Buck2 builds will generate a new version with actual git metadata.

/** Abbreviated commit hash (typically 8 characters) */
export const ABBREVIATED_COMMIT_HASH = "00000000";
/** Git branch name */
export const BRANCH = "unknown";
/** Calendar version in `YYYYMMDD.hhmmss.0` format (e.g., "20241127.123456.0") */
export const CAL_VER = "0.0.0";
/** Commit date in strict ISO 8601 format (e.g., "2024-11-27T12:34:56Z") */
export const COMMITTER_DATE_ISO8601 = "1970-01-01T00:00:00Z";
/** Commit date as Unix timestamp (seconds since epoch) */
export const COMMITTER_DATE_TIMESTAMP = 0;
/** Full 40-character commit SHA-1 hash */
export const COMMIT_HASH = "0000000000000000000000000000000000000000";
/** Whether the working tree had uncommitted changes at build time */
export const IS_DIRTY = true;
/**
 * Canonical version string combining calendar version and commit hash (e.g.,
 * "20241127.123456.0-sha.a1b2c3d")
 */
export const VERSION = "0.0.0-dev";

/**
 * Generates the User-Agent string for HTTP requests and analytics.
 *
 * Format: si-cli/{VERSION} ({arch}-{os}; {kernel_version})
 * Example: si-cli/0.0.0-dev (aarch64-darwin; 14.5.0)
 *
 * @returns User-Agent string
 */
export function getUserAgent(): string {
  return `si-cli/${VERSION} (${Deno.build.arch}-${Deno.build.os}; ${Deno.osRelease()})`;
}
