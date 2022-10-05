async function createKeyPair(component_view) {
    console.log(component_view);

    // TODO(nick): these checks should be removed once "all qualifications must pass before fixing" is implemented.
    const region = component_view.data.properties.region;
    if (region === undefined || region === null || region === "") {
        throw new Error("region must be set");
    }
    const keyName = component_view.data.properties.keyName;
    if (keyName === undefined || keyName === null || keyName === "") {
        throw new Error("keyName must be set");
    }
    const keyType = component_view.data.properties.keyType;
    if (keyType === undefined || keyType === null || keyType === "") {
        throw new Error("keyType must be set");
    }
    const tags = component_view.data.properties.tags;

    // Initialize the input JSON.
    let jsonBuilder = {
        "KeyName": `"${keyName}"`,
        "KeyType": `"${keyType}"`
    };

    // Add tags to the input JSON if we find any.
    let jsonTagsBuilder = [];
    for (const key of tags) {
        jsonTagsBuilder.push({
            "Key": `"${key}"`,
            "Value": `"${tags[key]}"`,
        })
    }
    if (jsonTagsBuilder.length > 0) {
        jsonBuilder["TagSpecifications"] = [
            {
                "Tags": jsonTagsBuilder
            }
        ];
    }

    // Now, create the key pair.
    const cliInputJson = JSON.stringify(jsonBuilder);
    const child = await siExec.waitUntilEnd("aws", [
        "ec2",
        "create-key-pair",
        "--region",
        region,
        "--cli-input-json",
        `'${cliInputJson}'`
    ]);
    if (child.exitCode !== 0) {
        throw new Error(child.stderr);
    }

    // We will likely want the "KeyPairId" field off the output.
    console.log(child.stdout);
    return JSON.parse(child.stdout);
}
