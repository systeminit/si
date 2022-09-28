async function exists(component, resource) {
  if (resource === undefined || resource === null) {
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
