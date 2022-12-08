function generateYAML(input) {
  return {
    format: "yaml",
    code: Object.keys(input.domain).length > 0 ? YAML.stringify(input.domain) : ""
  };
}
