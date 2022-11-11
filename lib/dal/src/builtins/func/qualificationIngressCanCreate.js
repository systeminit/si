async function qualification(component) {
  // Initialize the input JSON.
  const object = {
    "GroupId": component.data.properties.domain.GroupId,
     "IpPermissions": [],
  };

  for (const value of component.data.properties.domain.IpPermissions) {
    object["IpPermissions"].push({
      "FromPort": parseInt(value.FromPort),
      "ToPort": parseInt(value.ToPort),
      "IpProtocol": value.IpProtocol,
      "IpRanges": [ { "CidrIp": value.CidrIp } ],
    });
  }

  // Normalize tags to be in the weird Map-like structure AWS uses (array of { Key: string, Value: string } where Key is unique
  const tags = [];
  for (const [key, value] of Object.entries(component.data.properties.domain.tags)) {
    tags.push({
      "Key": key,
      "Value": value,
    });
  }
  if (tags.length > 0) {
    object["TagSpecifications"] = [{
      "ResourceType": component.data.properties.domain.awsResourceType,
      "Tags": tags
    }];
  }

  // Now, dry-run creation of the ingress
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "authorize-security-group-ingress",
    "--region",
    component.data.properties.domain.region,
    "--dry-run",
    "--cli-input-json",
    JSON.stringify(object),
  ]);

  // We have to use `includes` instead of `startsWith` because the line can start with a line feed char
  const success = child.stderr.includes('An error occurred (DryRunOperation)');

  return {
    qualified: success,
    message: success ? 'Component qualified' : child.stderr,
  }
}
