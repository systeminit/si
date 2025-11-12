/**
 * Filters component attributes to only relevant paths.
 * Excludes /resource/payload as it's extracted separately.
 *
 * @param attributes - Raw component attributes
 * @returns Filtered attributes with full paths as keys
 */
export function filterAttributes(
  attributes: { [key: string]: unknown },
): Record<string, unknown> {
  const filtered: Record<string, unknown> = {};

  for (const [path, value] of Object.entries(attributes)) {
    // Exclude /resource/payload as it's extracted as a separate field
    if (path === "/resource/payload") {
      continue;
    }

    if (
      path.startsWith("/si/") ||
      path.startsWith("/domain/") ||
      path.startsWith("/secrets/") ||
      path.startsWith("/resource_value/")
    ) {
      filtered[path] = value;
    }
  }

  return filtered;
}
