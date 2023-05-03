async function refresh(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "ok",
      message: component.properties.resource?.message
    };
  }

  const ruleId = resource.SecurityGroupRuleId;
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-security-group-rules",
    "--security-group-rule-ids",
    resource.SecurityGroupRuleId,
    "--region",
    component.properties.domain.region,
  ]);

  if (child.stderr.includes("InvalidSecurityGroupRuleId.NotFound")) {
    console.log(`Security Group Rule ID: ${ruleId}`);
    console.error(child.stderr);
    return {
      status: "error",
      message: `Security Group Rule not found (InvalidSecurityGroupRuleId.NotFound)`,
    }
  }

  if (child.stderr.includes("InvalidSecurityGroupRuleId.Malformed")) {
    console.log(`Security Group Rule ID: ${ruleId}`);
    console.error(child.stderr);
    return {
      status: "error",
      payload: resource,
      message: "Security Group Rule Id is invalid (InvalidSecurityGroupRuleId.Malformed)",
    }
  }

  if (child.exitCode !== 0) {
    console.log(`Security Group Rule ID: ${ruleId}`);
    console.error(child.stderr);
    return {
      status: "error",
      payload: resource,
      message: `AWS CLI 2 "aws ec2 describe-security-group-rules" returned non zero exit code (${child.exitCode})`,
    }
  }

  const rule = JSON.parse(child.stdout).SecurityGroupRules[0];
  if (rule.IsEgress) {
    return { status: "error", message: "Incorrect ingress rule found" };
  }

  return { payload: rule, status: "ok" };
}
