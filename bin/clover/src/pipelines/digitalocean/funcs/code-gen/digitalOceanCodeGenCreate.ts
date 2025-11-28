async function main(component: Input): Promise<Output> {
  // Extract only the properties that should be used for creation
  const payload = _.cloneDeep(component.domain);

  // Remove the 'extra' metadata object that SI uses internally
  delete payload.extra;

  // Visit the prop tree and remove empty/null values
  const cleaned = extLib.removeEmpty(payload);

  return {
    format: "json",
    code: JSON.stringify(cleaned, null, 2),
  };
}
