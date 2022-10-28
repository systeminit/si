async function create(component) {
    if (component.resource?.data) {
        throw new Error("resource already exists");
    }

    // Initialize the input JSON.
    const object = {
        "Description": component.properties.domain.Description,
        "GroupName": component.properties.domain.GroupName,
        "VpcId": component.properties.domain.VpcId,
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

    // Now, create the security group.
    const child = await siExec.waitUntilEnd("aws", [
        "ec2",
        "create-security-group",
        "--region",
        component.properties.domain.region,
        "--cli-input-json",
        JSON.stringify(object),
    ]);

    if (child.exitCode !== 0) {
        throw new Error(`Failure running aws ec2 create-security-group (${child.exitCode}): ${child.stderr}`);
    }

    return { value: JSON.parse(child.stdout) };
}
