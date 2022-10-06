async function deleteTagsForKeyPair(component_view) {
    console.log(component_view);

    // TODO(nick): these checks should be removed once "all qualifications must pass before fixing" is implemented.
    const region = component_view.data.properties.region;
    if (region === undefined || region === null || region === "") {
        throw new Error("region must be set");
    }
    const resources = component_view.data.resources;
    if (resources === undefined || resources === null || resources.length < 1) {
        // The "--resources" flag is required and the tag will not exist for resources
        // corresponding to this component if the resources do not exist. It's like a
        // reference counter: no resources, no tags for that resource. Thus, we fail
        // here instead of returning early or proceeding with intent to "do nothing".
        throw new Error("at least one resource must exist");
    }

    const componentTags = component_view.data.properties.tags;
    if (componentTags !== undefined && componentTags !== null && componentTags.length > 0) {
        // If there are tags, let's delete tags on a "per resource" basis. We do this because not all resources will
        // necessarily match the model.
        for (const resource of resources) {
            const resourceTags = resources.data["Tags"];

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
                `${resources.data["KeyPairId"]}`,
                "--cli-input-json",
                cliInputJson
            ]);
            if (child.exitCode !== 0) {
                throw new Error(child.stderr);
            }
        }
    } else {
        // Otherwise, the component tags are empty, and we can delete them all.
        let spaceSeperatedAwsResources = "";
        for (const resource of resources) {
            if (spaceSeperatedAwsResources === "") {
                spaceSeperatedAwsResources = `"${resources.data["KeyPairId"]}"`;
            } else {
                spaceSeperatedAwsResources = `${spaceSeperatedAwsResources} "${resources.data["KeyPairId"]}"`;
            }
        }

        const child = await siExec.waitUntilEnd("aws", [
            "ec2",
            "delete-tags",
            "--region",
            region,
            "--resources",
            spaceSeperatedAwsResources,
        ]);
        if (child.exitCode !== 0) {
            throw new Error(child.stderr);
        }
    }
}
