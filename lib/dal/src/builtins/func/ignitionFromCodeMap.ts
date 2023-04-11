async function ignitionFromCodeMap(input: Input): Promise<Output> {
  return input.code?.["si:generateButaneIgnition"]?.code;
}
