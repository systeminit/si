async function refresh(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "ok",
      message: component.properties.resource?.message
    };
  }

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-security-groups",
    "--group-ids",
    resource.GroupId,
    "--region",
    component.properties.domain.region,
  ]);

  if (child.stderr.includes("InvalidGroup.NotFound")) {
    console.log(`Group Id: ${resource.GroupId}`);
    console.error(child.stderr);
    return {
      status: "error",
      message: `Security Group not found (InvalidGroup.NotFound)`,
    }
  }

  if (child.stderr.includes("InvalidGroupId.Malformed")) {
    console.log(`Group Id: ${resource.GroupId}`);
    console.error(child.stderr);
    return {
      status: "error",
      payload: resource,
      message: "Security Group Id is invalid (InvalidGroupId.Malformed)",
    }
  }

  if (child.exitCode !== 0) {
    console.log(`Group Id: ${resource.GroupId}`);
    console.error(child.stderr);
    return {
      status: "error",
      payload: resource,
      message: `AWS CLI 2 "aws ec2 describe-security-groups" returned non zero exit code (${child.exitCode})`,
    }
  }

  const object = JSON.parse(child.stdout);

  if (!object.SecurityGroups || object.SecurityGroups.length === 0) {
    console.log(`Group Id: ${resource.GroupId}`);
    console.error(child.stdout);
    return {
      status: "error",
      payload: resource,
      message: "Security Group not found in payload returned by AWS, but it should be there",
    }
  }

  return { payload: object.SecurityGroups[0], status: "ok" };
}
