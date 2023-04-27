async function deleteResource(component: Input): Promise<Output> {
  const resource = component.properties.resource?.payload;
  // Now, delete the Ec2 Instance.
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "delete-key-pair",
    "--region",
    component.properties.domain.region,
    "--key-pair-id",
    resource.KeyPairId,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      payload: resource,
      message: `Unable to delete Key Pair, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    };
  }

  return { payload: null, status: "ok" };
}
