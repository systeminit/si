async function refresh(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "ok",
      message: component.properties.resource?.message,
    };
  }

  const ruleIds: number[] = resource.SecurityGroupRules.map(
    (x) => x.SecurityGroupRuleId,
  );

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-security-group-rules",
    "--security-group-rule-ids",
    ...ruleIds,
    "--region",
    component.properties.domain.region,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      payload: resource,
      message: `AWS CLI 2 "aws ec2 describe-security-group-rules" returned non zero exit code (${child.exitCode})`,
    };
  }

  const rules = JSON.parse(child.stdout);
  rules.SecurityGroupRules.forEach((x, i) => {
    if (x.isEgress) {
      return {
        status: "error",
        message: `expected Security Group Rule ID ${x.SecurityGroupRuleId} to be an ingress rule but is egress`,
      };
    }
  });

  return { payload: rules, status: "ok" };
}
