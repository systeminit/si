async function deleteResource(component) {
  const resource = component.properties.resource?.value;

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "revoke-security-group-egress",
    "--region",
    component.properties.domain.region,
    "--security-group-rule-ids",
    resource.SecurityGroupRuleId,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message: `Unable to delete Egress, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    }
  }

  return { value: null, status: "ok" };
}
