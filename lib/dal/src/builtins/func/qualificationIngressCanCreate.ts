async function qualification(input: Input): Promise<Output> {
  const code = input.code?.["si:generateAwsIngressJSON"]?.code;
  if (!code) {
    return {
      result: "failure",
      message: "component doesn't have JSON representation",
    };
  }

  if (!input.domain?.region) {
    return {
      result: "failure",
      message: "component doesn't have a region set",
    };
  }

  // Now, dry-run creation of the ingress
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "authorize-security-group-ingress",
    "--region",
    input.domain.region,
    "--dry-run",
    "--cli-input-json",
    code,
  ]);

  // We have to use `includes` instead of `startsWith` because the line can start with a line feed char
  const success = child.stderr.includes("An error occurred (DryRunOperation)");
  if (success && !input.domain?.GroupId) {
    return {
      result: "warning",
      message:
        "GroupId must be set. If a Security Group is connected to this component the id will be automatically set when the fix flow creates the security group after merging this change-set",
    };
  }

  return {
    result: success ? "success" : "failure",
    message: success ? "component qualified" : child.stderr,
  };
}
