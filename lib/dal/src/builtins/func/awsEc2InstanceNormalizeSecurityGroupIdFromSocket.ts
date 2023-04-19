async function securityGroups(input: Input): Promise<Output> {
  const value = input.value?.SecurityGroupId;
  if (value === undefined) return value;
  if (value === null) return value;
  return !Array.isArray(value) ? [value] : value;
}
