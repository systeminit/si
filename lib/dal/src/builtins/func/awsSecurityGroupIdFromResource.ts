async function parse(properties: Input): Promise<Output> {
  const SecurityGroupId = properties.resource?.value?.GroupId;
  const outdated = !!(input.resource?.value && !_.isEqual(input.domain, input.applied_model?.domain));
  return { SecurityGroupId, outdated };
}
