async function exists(component) {
  if (component.properties.resource === undefined
      || component.properties.resource === null
      || component.properties.resource === ""
      || component.properties.resource === "undefined"
      || component.properties.resource === "null") {
    return {
      success: false,
      recommendedActions: ["create"]
    }
  }
  return {
    success: true,
    recommendedActions: [],
  }
};
