function generateAwsJSON(component) {
  delete component.properties.domain.region
  return {
    format: "json",
    code: JSON.stringify(component.properties.domain)
  };
}
