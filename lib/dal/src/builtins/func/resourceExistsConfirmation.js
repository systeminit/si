async function exists(component) {
  if (!component.properties.resource?.value) {
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
