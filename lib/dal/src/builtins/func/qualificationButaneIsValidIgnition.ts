async function qualificationButaneIsValidIgnition(input: Input): Promise<Output> {
  if (!input.domain) {
    return {
      result: "failure",
      message: "domain is empty"
    }
  }
  const domainJson = JSON.stringify(input.domain);
  // NOTE(nick): this is where one would insert profanities. I'm reformed... right?
  domainJson.replace("\n", "\\\\n");
  const options = { input: `${domainJson}` };
  const child = await siExec.waitUntilEnd("butane", ["--pretty", "--strict"], options);
  return {
    result: child.exitCode === 0 ? "success" : "failure",
    // NOTE(nick): we probably want both stdout and stderr always, but this will suffice for now.
    message: child.exitCode === 0 ? child.stdout : child.stderr,
  };
}
