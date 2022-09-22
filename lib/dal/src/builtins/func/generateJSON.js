function generateJSON(component) {
  return {
    format: "json",
    code: JSON.stringify(component.properties.domain)
  };
}
