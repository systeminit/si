async function create(component: Input): Promise<Output> {
  if (component.properties.resource?.value) {
    return {
      status: "error",
      message: "Resource already exists",
      value: component.properties.resource.value,
    }
  }

  // Now, create the key pair.
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "create-key-pair",
    "--region",
    component.properties.domain.region,
    "--cli-input-json",
    component.properties.code["si:generateAwsKeyPairJSON"]?.code,
  ]);

  if (child.exitCode !== 0) {
    console.error(child.stderr);
    return {
      status: "error",
      message: `Unable to create Key Pair, AWS CLI 2 exited with non zero code: ${child.exitCode}`,
    }
  }

  return { value: JSON.parse(child.stdout), status: "ok" };
}
