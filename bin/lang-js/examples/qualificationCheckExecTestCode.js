// Encode this into Base64 with:
//
// ```sh
// cat examples/qualificationCheckExecTestCode.js | base64 | tr -d '\n'
// ```

async function exec(component) {
  const child = await siExec.waitUntilEnd("echo", [`Component ${component.data.name} says: this will be my longest yeaaahhhhhhhhhhh boyyyyyyyyyyy ever`]);
  return {
    qualified: child.exitCode === 0,
    message: child.stdout,
  };
}
