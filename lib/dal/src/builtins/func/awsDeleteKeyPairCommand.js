async function deleteKeyPair(component_view) {
  console.log(component_view);

  // TODO(nick): these checks should be removed once "all qualifications must pass before fixing" is implemented.
  const region = component_view.data.properties.region;
  if (region === undefined || region === null || region === "") {
    throw new Error("region must be set");
  }
  const resource = component_view.data.resource;
  if (resource === undefined || resource === null) {
    // The "--resources" flag is required.
    throw new Error("the resource must exist");
  }

  // Delete the key pair.
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "delete-key-pair",
    "--region",
    region,
    "--key-pair-id",
    `${resourcs.data["KeyPairId"]}`,
  ]);
  if (child.exitCode !== 0) {
    throw new Error(child.stderr);
  }
}
