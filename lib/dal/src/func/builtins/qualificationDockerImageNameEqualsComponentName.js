function dockerImageNameEqualsComponentName(component) {
  const result = component.name === component.properties.image;
  if (result) {
    return {
      qualified: result,
      subChecks: [
        {
          status: "Success",
          description: "dockerImageName (" + component.properties.image + ") is not equal to componentName (" + component.name + ")"
        }
      ],
    };
  }
  return {
    qualified: result,
    subChecks: [
      {
        status: "Failure",
        description: "dockerImageName (" + component.properties.image + ") is not equal to componentName (" + component.name + ")"
      }
    ],
  };
}
