async function create(component) {
    if (component.resource?.data) {
        throw new Error("resource already exists");
    }

    // Initialize the input JSON.
    const object = {
        "KeyName": component.properties.domain.KeyName,
        "KeyType": component.properties.domain.KeyType,
    };

    // Normalize tags to be in the weird Map-like structure AWS uses (array of { Key: string, Value: string } where Key is unique
    const tags = [];
    for (const [key, value] of Object.entries(component.properties.domain.tags)) {
        tags.push({
            "Key": key,
            "Value": value,
        });
    }
    if (tags.length > 0) {
        object["TagSpecifications"] = [{
            "ResourceType": component.properties.domain.awsResourceType,
            "Tags": tags
        }];
    }

    // Now, create the key pair.
    const child = await siExec.waitUntilEnd("aws", [
        "ec2",
        "create-key-pair",
        "--region",
        component.properties.domain.region,
        "--cli-input-json",
        JSON.stringify(object),
    ]);

    if (child.exitCode !== 0) {
        throw new Error(`Failure running aws ec2 create-key-pair (${child.exitCode}): ${child.stderr}`);
    }

    console.log(child.stdout);
    return { value: JSON.parse(child.stdout) };
}
