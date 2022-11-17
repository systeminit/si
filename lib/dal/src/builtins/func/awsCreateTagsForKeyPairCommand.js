async function createTagsForKeyPair(component_view) {
  console.log(component_view);

  // TODO(nick): these checks should be removed once "all qualifications must pass before fixing" is implemented.
  const region = component_view.data.properties.region;
  if (region === undefined || region === null || region === "") {
    throw new Error("region must be set");
  }
  const tags = component_view.data.properties.tags;
  if (tags === undefined || tags === null || tags.length < 1) {
    throw new Error("at least one tag must exist");
  }
  const resource = component_view.data.resource;
  if (resource === undefined || resource === null) {
    // The "--resources" flag is required.
    throw new Error("the resource must exist");
  }

  // Gather all AWS tags via the SI component properties.
  let cliInputJsonTags = [];
  for (const key of tags) {
    cliInputJsonTags.push({
      "Key": `${key}`,
      "Value": `${tags[key]}`,
    })
  }
  const cliInputJson = JSON.stringify(cliInputJsonTags);

  // Now, create all tags for the resource.
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "create-tags",
    "--region",
    region,
    "--resources",
    `"${resource.data["KeyPairId"]}"`,
    "--cli-input-json",
    cliInputJson
  ]);
  if (child.exitCode !== 0) {
    throw new Error(child.stderr);
  }
}
