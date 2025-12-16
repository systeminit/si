import { VERSION } from "./git_metadata.ts";

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
