function generateYAML(component) {
  return {
    format: "yaml",
    // TODO: do we want the whole component, just the properties, is there a need for value mapping or is this correct?
    code: YAML.stringify(component),
  };
}
