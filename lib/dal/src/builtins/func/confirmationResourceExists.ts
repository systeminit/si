async function exists(input: Input): Promise<Output> {
  if (!input.resource?.value) {
    return {
      success: false,
      recommendedActions: ["create"]
    }
  }
  return {
    success: true,
    recommendedActions: [],
  }
}
