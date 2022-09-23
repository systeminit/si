async function qualification(component) {
  let code;
  for (const codeObj of component.codes) {
    if (codeObj.format === "json") {
      code = codeObj.code;
    }
  }

  if (!code) {
    return {
      qualified: false,
      message: "Component doesn't have JSON representation"
    }
  }

  const dryRunStatus = await siExec.waitUntilEnd("aws", ["ec2", "run-instances", "--dry-run", "--cli-input-json", code])

  console.log(dryRunStatus.stderr);

  // We have to use `includes` instead of `startsWith` because the line can start with a line feed char
  const success = dryRunStatus.stderr.includes('An error occurred (DryRunOperation)');

  return {
    qualified: success,
    message: success ? 'Component qualified' : dryRunStatus.shortMessage
  }
}
