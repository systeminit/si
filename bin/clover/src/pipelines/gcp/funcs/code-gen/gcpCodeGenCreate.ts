async function main(component: Input): Promise<Output> {
  // Extract only the properties that should be used for creation
  const payload = _.cloneDeep(component.domain);

  // Remove the 'extra' metadata object that SI uses internally
  // But first extract PropUsageMap to know which props to exclude
  const propUsageMapJson = payload.extra?.PropUsageMap;
  delete payload.extra;

  // Remove excluded properties (props that are only used for URL construction, not request body)
  if (propUsageMapJson) {
    try {
      const propUsageMap = JSON.parse(propUsageMapJson);
      if (Array.isArray(propUsageMap.excluded)) {
        for (const excludedProp of propUsageMap.excluded) {
          delete payload[excludedProp];
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
