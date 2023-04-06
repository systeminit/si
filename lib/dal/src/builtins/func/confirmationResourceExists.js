async function exists(input) {
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
