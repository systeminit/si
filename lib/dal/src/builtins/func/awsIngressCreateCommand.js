async function create(component) {
    if (component.resource?.data) {
        throw new Error("resource already exists");
    }

    // Initialize the input JSON.
    const object = {
        "GroupId": component.properties.domain.GroupId,
	"IpPermissions": [],
    };

    for (const value of component.properties.domain.IpPermissions) {
        object["IpPermissions"].push({
            "FromPort": value.FromPort,
            "ToPort": value.ToPort,
            "IpProtocol": value.IpProtocol,
            "IpRanges": [ { "CidrIp": value.CidrIp } ],
        });
    }

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

    // Now, create the ingress
    const child = await siExec.waitUntilEnd("aws", [
        "ec2",
        "authorize-security-group-ingress",
        "--region",
        component.properties.domain.region,
        "--cli-input-json",
        JSON.stringify(object),
    ]);

    if (child.exitCode !== 0) {
        throw new Error(`Failure running aws ec2 authorize-security-group-ingress (${child.exitCode}): ${child.stderr}`);
    }

    return { value: JSON.parse(child.stdout) };
}
