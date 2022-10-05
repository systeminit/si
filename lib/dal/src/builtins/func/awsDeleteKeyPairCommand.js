async function deleteKeyPair(component_view) {
    console.log(component_view);

    // TODO(nick): these checks should be removed once "all qualifications must pass before fixing" is implemented.
    const region = component_view.data.properties.region;
    if (region === undefined || region === null || region === "") {
        throw new Error("region must be set");
    }
    const resources = component_view.data.resources;

    // Delete each key pair (should only be one since we are bound by region).
    // If there are no resources, we do nothing.
    let child_errors = [];
    for (const resource of resources) {
        const child = await siExec.waitUntilEnd("aws", [
            "ec2",
            "delete-key-pair",
            "--region",
            region,
            "--key-pair-id",
            `${resources.data["KeyPairId"]}`,
        ]);
        if (child.exitCode !== 0) {
            child_errors.push(child.stderr);
        }
    }
    if (child_errors.length > 0) {
        throw new Error(JSON.stringify(child_errors));
    }
}
