async function deleteTagsForKeyPair(component_view) {
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

  const componentTags = component_view.data.properties.tags;
  if (componentTags !== undefined && componentTags !== null && componentTags.length > 0) {
    const resourceTags = resource.data["Tags"];

    // Only delete tags that do not exist in the "componentTags", but _do_ exist on the resource.
    const tagsToDelete = resourceTags.filter(resourceTag => !componentTags.some(componentTag => componentTag["Key"] === resourceTag["Key"]));

    let jsonTagsBuilder = [];
    for (const key of tagsToDelete) {
      jsonTagsBuilder.push({
        "Key": `${key}`,
        "Value": `${resourceTags[key]}`,
      })
    }
    const cliInputJson = JSON.stringify({
      "Tags": jsonTagsBuilder
    });

    const child = await siExec.waitUntilEnd("aws", [
      "ec2",
      "delete-tags",
      "--region",
      region,
      "--resources",
      `${resource.data["KeyPairId"]}`,
      "--cli-input-json",
      cliInputJson
    ]);
    if (child.exitCode !== 0) {
      throw new Error(child.stderr);
    }
  } else {
    // Otherwise, the component tags are empty, and we can delete them all.
    const child = await siExec.waitUntilEnd("aws", [
      "ec2",
      "delete-tags",
      "--region",
      region,
      "--resources",
      `"${resource.data["KeyPairId"]}"`,
    ]);
    if (child.exitCode !== 0) {
      throw new Error(child.stderr);
    }
  }
}
