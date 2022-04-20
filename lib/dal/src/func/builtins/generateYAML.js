function generateYAML(component) {
  return {
    format: "yaml",
    code: Object.keys(component.properties.domain).length > 0 ?
      YAML.stringify(component.properties.domain) : ""
  };
}
