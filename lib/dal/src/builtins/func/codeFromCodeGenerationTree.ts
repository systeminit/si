async function codeFromCodeGenerationTree(input: Input): Promise<Output> {
  return input.tree.code;
}
