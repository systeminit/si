async function qualification(component) {
  // Initialize the input JSON.
  const object = {
      "Description": component.data.properties.domain.Description,
      "GroupName": component.data.properties.domain.GroupName,
      "VpcId": component.data.properties.domain.VpcId,
  };

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

  if (!component.data.properties.domain.region) {
    return {
      qualified: false,
      message: "Component doesn't have a region set"
    }
  }

  // Now, create the security group.
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "create-security-group",
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
