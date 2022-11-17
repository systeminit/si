function generateJSON(component) {
  // Initialize the input JSON.
  const object = {
    "ImageIds": [component.properties.domain.ImageId],
  };

  return {
    format: "json",
    code: JSON.stringify(object, null, '\t')
  };
}
