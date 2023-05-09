async function generateYAML(input: Input): Promise<Output> {
  return {
    format: "yaml",
    code:
      Object.keys(input.domain).length > 0 ? YAML.stringify(input.domain) : "",
  };
}
