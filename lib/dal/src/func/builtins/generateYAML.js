function generateYAML(component) {
  return {
    format: "yaml",
    code: YAML.stringify(component.properties),
  };
}
