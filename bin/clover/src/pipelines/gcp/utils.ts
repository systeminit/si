import { NormalizedGcpSchema } from "./schema.ts";

/** Remove "API", version numbers, and roman numerals from a GCP API title */
export function cleanGcpApiTitle(title: string): string {
  return title
    .replace(/\s+API\s*$/i, "")
    .replace(/\s+API\s+/gi, " ")
    .replace(/\s+v\d+$/i, "")
    .replace(/\s+I{1,3}$/i, "")
    .trim();
}

/** Remove duplicate "Cloud" words (e.g., "Google Cloud Cloud Storage" -> "Google Cloud Storage") */
export function deduplicateCloudWord(str: string): string {
  return str.replace(/\bCloud\s+Cloud\b/gi, "Cloud");
}

/** Title-case each segment and join with dots */
export function titleCaseResourcePath(segments: string[]): string {
  return segments
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(".");
}

/** Build full GCP type name from API title and resource path */
export function buildGcpTypeName(
  title: string,
  resourcePath: string[],
): string {
  const cleanTitle = cleanGcpApiTitle(title);
  const fullResourceName = titleCaseResourcePath(resourcePath);
  return deduplicateCloudWord(`Google Cloud ${cleanTitle} ${fullResourceName}`);
}

/** Build GCP category name from API title */
export function buildGcpCategory(title: string): string {
  return deduplicateCloudWord(`Google Cloud ${cleanGcpApiTitle(title)}`);
}

/**
 * Detects createOnly properties from their descriptions in the insert request schema.
 *
 * GCP Discovery Documents don't consistently mark immutable fields, so we pattern-match
 * against property descriptions to identify fields that cannot be updated after creation.
 *
 * @param insertRequestSchema - The schema for the insert/create request
 * @returns Array of property names that are createOnly
 */
export function detectCreateOnlyProperties(
  insertRequestSchema: NormalizedGcpSchema | undefined,
): string[] {
  const createOnlyProps: string[] = [];

  if (!insertRequestSchema?.properties) {
    return createOnlyProps;
  }

  for (
    const [propName, propDef] of Object.entries(insertRequestSchema.properties)
  ) {
    const rawDescription = propDef?.description || "";
    // Normalize newlines to spaces so patterns work across line breaks
    const description = rawDescription.replace(/\n/g, " ");

    // Match various patterns that indicate a field cannot be updated:
    // Patterns found across GCP Discovery Documents (aiplatform, compute, container, etc.):
    // 1. "immutable" - e.g., "It's immutable", "Immutable. A resource name", "remains immutable"
    // 2. "set only at resource creation" - e.g., "This field can be set only at resource creation time"
    // 3. "set at resource creation" - e.g., "This field is set at resource creation time"
    // 4. "specified during creation" - e.g., "must be specified during creation"
    // 5. "cannot be changed/modified/updated" - e.g., "cannot be changed after the resource is created"
    // 6. "must be set on creation and cannot be changed"
    //
    // Note: We are conservative and only match explicit immutability indicators.
    // Phrases like "provide when you create" are NOT considered createOnly because they may just
    // indicate the field should be provided at creation time, not that it's immutable.
    // We also check if description does NOT contain "and updated" or "and changed" to avoid
    // false positives like "can be set at creation and updated using patch"
    const hasCreateOnlyPattern =
      /\bimmutable\b|((can|must) be )?set (only )?(at|on|during).*(resource )?(creation|creat)|((can|could|must) be )?specified (only )?(at|on|during).*(resource )?(creation|creat)|cannot be (chang|modif|updat)|must be set (at|on) creation and cannot be chang/i
        .test(description);
    const hasUpdateablePattern =
      /\b(and|or) (be )?(updated|changed|modified)\b/i.test(description);
    const isCreateOnly = hasCreateOnlyPattern && !hasUpdateablePattern;

    if (isCreateOnly) {
      createOnlyProps.push(propName);
    }
  }

  return createOnlyProps;
}
