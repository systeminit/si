async function exists(component) {
  if (component.resource === undefined || component.resource === null
      || component.resource.data === undefined || component.resource.data === null) {
    return {
      success: true,
      recommendedActions: ["create"]
    }
  }
  return {
    success: true,
    recommendedActions: [],
  }
};
