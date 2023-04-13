async function parse(properties: Input): Promise<Output> {
  return properties.resource?.value?.GroupId;
}
