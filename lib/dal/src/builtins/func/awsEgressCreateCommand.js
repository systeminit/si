async function create(component) {
    if (component.resource?.data) {
        throw new Error("resource already exists");
    }

    // Initialize the input JSON.
    const object = {
        "GroupId": component.properties.domain.GroupId,
        "FromPort": parseInt(value.FromPort),
        "ToPort": parseInt(value.ToPort),
        "IpProtocol": value.IpProtocol,
        "CidrIp": value.CidrIp,
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

    // Now, create the egress
    const child = await siExec.waitUntilEnd("aws", [
        "ec2",
        "authorize-security-group-egress",
        "--region",
        component.properties.domain.region,
        "--cli-input-json",
        JSON.stringify(object),
    ]);

    if (child.exitCode !== 0) {
        throw new Error(`Failure running aws ec2 authorize-security-group-egress (${child.exitCode}): ${child.stderr}`);
    }

    console.log(child.stdout);
    return { value: JSON.parse(child.stdout) };
}
