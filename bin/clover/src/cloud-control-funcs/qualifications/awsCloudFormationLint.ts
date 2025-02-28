async function main(component: Input): Promise<Output> {
  const codeString = component.code?.["awsCloudFormationLint"]?.code;
  await Deno.writeTextFile("/tmp/cfn.json", codeString);
  const args = [
    "-f",
    "pretty",
  ];
  if (component.domain?.extra?.Region) {
    args.push("-r");
    args.push(component.domain?.extra?.Region);
  }
  args.push("-t");
  args.push("/tmp/cfn.json");

  const child = await siExec.waitUntilEnd("cfn-lint", args);
  console.log(child.stdout);

  let result = "success";
  let message = child.stdout;
  // is an error
  if (child.exitCode === 1) {
    result = "failure";
    message = `cfn-lint failed: \n\n${child.stdout}\n\n${child.stderr}`;
  } else if (child.exitCode === 2) {
    result = "failure";
    // is a warning
  } else if (child.exitCode === 4) {
    result = "warning";
    // is an error & warning
  } else if (child.exitCode === 6) {
    result = "failure";
    // is informational
  } else if (child.exitCode === 8) {
    result = "warning";
    // is an error & informational
  } else if (child.exitCode === 10) {
    result = "failure";
    // is a warning & informational
  } else if (child.exitCode === 12) {
    result = "warning";
    // is an error & warning & informational
  } else if (child.exitCode === 14) {
    result = "failure";
  } else if (child.exitCode !== 0) {
    result = "failure";
    message =
      `cfn-lint failed: \n\n${child.stdout}\n\n${child.stderr}\n\n${child.shortMessage}`;
  }

  return {
    result,
    message,
  };
}
