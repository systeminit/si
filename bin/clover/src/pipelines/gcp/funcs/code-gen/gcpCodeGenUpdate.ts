async function main(component: Input): Promise<Output> {
  // For GCP updates, we generate all fields here for preview
  // The actual filtering to only changed fields happens in the update action
  const payload = _.cloneDeep(component.domain);

  // Remove the 'extra' metadata object that SI uses internally
  // But first extract PropUsageMap to know which props to exclude
  const propUsageMapJson = payload.extra?.PropUsageMap;
  delete payload.extra;

  // Remove properties that shouldn't be in the request body
  if (propUsageMapJson) {
    try {
      const propUsageMap = JSON.parse(propUsageMapJson);

      // Remove excluded properties (props that are only used for URL construction, not request body)
      if (Array.isArray(propUsageMap.excluded)) {
        for (const excludedProp of propUsageMap.excluded) {
          delete payload[excludedProp];
        }
      }

      // Remove path parameters (like 'parent', 'project', etc.)
      // These are used in the URL path, not the request body
      if (Array.isArray(propUsageMap.pathParameters)) {
        for (const pathParam of propUsageMap.pathParameters) {
          delete payload[pathParam];
        }
      }

      if (Array.isArray(propUsageMap.createOnly)) {
        for (const createOnlyPath of propUsageMap.createOnly) {
          // Convert /domain/propertyName to propertyName
          const propName = createOnlyPath.replace(/^\/domain\//, "");
          delete payload[propName];
        }
      }
    } catch {
      // If parsing fails, continue without filtering
    }
  }

  // Visit the prop tree and remove empty/null values
  const cleaned = extLib.removeEmpty(payload);

  return {
    format: "json",
    code: JSON.stringify(cleaned, null, 2),
  };
}
