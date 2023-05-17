async function deleteResource(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;
  // Now, delete the EIP.
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "release-address",
    "--region",
    component.properties.domain.region,
    "--allocation-id",
    resource.AllocationId,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      payload: resource,
      message: `Unable to delete Elastic IP, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    };
  }

  return { payload: null, status: "ok" };
}
``;
