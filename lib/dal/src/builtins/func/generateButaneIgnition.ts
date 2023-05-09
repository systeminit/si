async function generateButaneIgnition(input: Input): Promise<Output> {
  const domainJson = JSON.stringify(input.domain);
  domainJson.replace("\n", "\\\\n");
  const options = { input: `${domainJson}` };
  const { stdout } = await siExec.waitUntilEnd(
    "butane",
    ["--pretty", "--strict"],
    options,
  );

  return {
    format: "json",
    code: stdout.toString(),
  };
}
