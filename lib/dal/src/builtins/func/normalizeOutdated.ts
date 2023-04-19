async function normalizeOutdated(input: Input): Promise<Output> {
  const value = input.value?.outdated;
  if (value === undefined) return value;
  if (value === null) return value;
  return !Array.isArray(value) ? [value] : value;
}
