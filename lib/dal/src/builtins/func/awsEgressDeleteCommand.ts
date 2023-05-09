async function deleteResource(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;

  const ruleIds: number[] = resource.SecurityGroupRules.map(
    (x) => x.SecurityGroupRuleId,
  );
  const groupId = resource.SecurityGroupRules[0].GroupId;

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "revoke-security-group-egress",
    "--region",
    component.properties.domain.region,
    "--security-group-rule-ids",
    ...ruleIds,
    "--group-id",
    groupId,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      payload: resource,
      message: `Unable to delete Egress Rule, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    };
  }

  return { payload: null, status: "ok" };
}
