async function refresh(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;
  if (!resource) {
    return {
      status: component.properties.resource?.status ?? "ok",
      message: component.properties.resource?.message,
    };
  }

  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "describe-addresses",
    "--allocation-ids",
    resource.AllocationId,
    "--region",
    component.properties.domain.region,
  ]);

  if (child.exitCode !== 0) {
    console.log(`EIP Allocation ID: ${resource.AllocationId}`);
    console.error(child.stderr);
    return {
      payload: resource,
      status: "error",
      message: `AWS CLI 2 "aws ec2 describe-addresses" returned non zero exit code (${child.exitCode})`,
    };
  }

  const object = JSON.parse(child.stdout);

  return { payload: object.Addresses[0], status: "ok" };
}
