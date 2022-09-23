async function qualification(component) {
  const {
    data: {
      properties: { // Derived from the fields in the Attributes panel
        domain: {
          region
        },
      },
      kind, // Standard | Credential
      system, // Can be null
    },
    parents, // An array with this component's parents. Parent's parents aren't accessible
    codes
    } = component;
  
  let code;
  for (const codeObj of codes) {
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

  if (!region) {
    return {
      qualified: false,
      message: "Component doesn't have a region set"
    }
  }

  const dryRunStatus = await siExec.waitUntilEnd("aws", ["ec2", "run-instances",
    "--region", region, "--dry-run", "--cli-input-json", code])

  console.log(dryRunStatus.stderr);

  // We have to use `includes` instead of `startsWith` because the line can start with a line feed char
  const success = dryRunStatus.stderr.includes('An error occurred (DryRunOperation)');

  return {
    qualified: success,
    message: success ? 'Component qualified' : dryRunStatus.shortMessage
  }
}
