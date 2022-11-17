async function generateButaneIgnition(component) {
  const domainJson = JSON.stringify(component.properties.domain);
  domainJson.replace("\n", "\\\\n");
  const options = { input: `${domainJson}` };
  const { stdout } = await siExec.waitUntilEnd("butane", ["--pretty", "--strict"], options);
  return {
    format: "json",
    code: stdout.toString(),
  };
}
