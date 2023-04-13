async function qualification(input: Input): Promise<Output> {
  const code = input.code?.["si:generateAwsEc2JSON"]?.code;
  if (!code) {
    return {
      result: "failure",
      message: "component doesn't have JSON representation"
    }
  }

  if (!input.domain.region) {
    return {
      result: "failure",
      message: "component doesn't have a region set"
    }
  }

  const dryRunStatus = await siExec.waitUntilEnd("aws", [
    "ec2",
    "run-instances",
    "--region",
    input.domain.region,
    "--dry-run",
    "--cli-input-json",
    code
  ]);

  console.log(dryRunStatus.stderr);

  if (dryRunStatus.stderr.includes("An error occurred (InvalidKeyPair.NotFound)")) {
    return {
      result: "warning",
      message: "Key Pair must exist. It will be created by the fix flow after merging this change-set"
    };
  }

  // We have to use `includes` instead of `startsWith` because the line can start with a line feed char
  const success = dryRunStatus.stderr.includes('An error occurred (DryRunOperation)');

  return {
    result: success ? "success" : "failure",
    message: success ? 'component qualified' : dryRunStatus.shortMessage
  }
}
