function metadata(input: Input): Promise<Output> {
  const outdated = !!(input.resource?.value && !_.isEqual(input.domain, input.applied_model?.domain));
  return { KeyName: input.KeyName, outdated };
}
