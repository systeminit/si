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
    const resources = component_view.data.resources;
    if (resources === undefined || resources === null || resources.length < 1) {
        // The "--resources" flag is required.
        throw new Error("at least one resource must exist");
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

    // Gather all AWS resources via the SI resources.
    let spaceSeperatedAwsResources = "";
    for (const resource of resources) {
        if (spaceSeperatedAwsResources === "") {
            spaceSeperatedAwsResources = `"${resources.data["KeyPairId"]}"`;
        } else {
            spaceSeperatedAwsResources = `${spaceSeperatedAwsResources} "${resources.data["KeyPairId"]}"`;
        }
    }

    // Now, create all tags for all resources.
    const child = await siExec.waitUntilEnd("aws", [
        "ec2",
        "create-tags",
        "--region",
        region,
        "--resources",
        spaceSeperatedAwsResources,
        "--cli-input-json",
        cliInputJson
    ]);
    if (child.exitCode !== 0) {
        throw new Error(child.stderr);
    }
}
