async function qualification(component) {
  // Initialize the input JSON.
  const object = {
    "KeyName": component.data.properties.domain.KeyName,
    "KeyType": component.data.properties.domain.KeyType,
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

  // Now, creation of the key pair.
  const child = await siExec.waitUntilEnd("aws", [
    "ec2",
    "create-key-pair",
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
