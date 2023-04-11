async function deleteResource(component: Input): Promise<Output> {
  const resource = component.properties.resource?.value;

  const instances = Array.isArray(resource) ? resource : [resource];
  const instanceIds = instances.flatMap((i) => i.Instances).map((i) => i.InstanceId).filter((id) => !!id);
  if (!instanceIds || instanceIds.length === 0) return {
    status: "error",
    value: resource,
    message: "No EC2 instance id found"
  };

  // Now, delete the Ec2 Instance.
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "terminate-instances",
    "--region",
    component.properties.domain.region,
    "--instance-ids",
    ...instanceIds,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      value: resource,
      message: `Unable to delete Ec2 Instance, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    };
  }

  return { value: null, status: "ok" };
}
