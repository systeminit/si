async function main(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "ok",
      message: component.properties.resource?.message,
    };
  }

  if (!resource.InstanceId) {
    return {
      status: "error",
      payload: resource,
      message: "No EC2 instance id found",
    };
  }

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "stop-instances",
    "--instance-ids",
    resource.InstanceId,
    "--region",
    component.properties.domain?.extra.Region,
  ]);

  if (child.exitCode !== 0) {
    console.log(`Ec2 Instance ID: ${resource.InstanceId}`);
    console.error(child.stderr);
    return {
      payload: resource,
      status: "error",
      message:
        `AWS CLI 2 "aws ec2 stop-instances" returned non zero exit code (${child.exitCode})`,
    };
  }

  return {
    payload: resource,
    status: "ok",
  };
}
