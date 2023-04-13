async function normalize(input: Input): Promise<Output> {
  if (input.value === undefined) return input.value;
  if (input.value === null) return input.value;
  return !Array.isArray(input.value) ? [input.value] : input.value;
}
