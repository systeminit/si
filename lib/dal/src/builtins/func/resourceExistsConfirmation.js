async function exists(component) {
  if (component.resource === undefined || component.resource === null) {
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
