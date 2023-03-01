async function deleteResource(component) {
  const resource = component.properties.resource?.value;
  // Now, delete the security group.
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "delete-security-group",
    "--region",
    component.properties.domain.region,
    "--group-id",
    resource.GroupId,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message: `Unable to delete Security Group, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    }
  }

  return { value: null, status: "ok" };
}
