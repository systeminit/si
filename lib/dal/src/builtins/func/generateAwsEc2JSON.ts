async function generateAwsEc2JSON(input: Input): Promise<Output> {
  // Initialize the input JSON.
  const object = {
    "ImageId": input.domain.ImageId,
    "InstanceType": input.domain.InstanceType,
    "KeyName": input.domain.KeyName,
    "SecurityGroupIds": input.domain.SecurityGroupIds,
    "UserData": input.domain.UserData,
  };

  // Normalize tags to be in the weird Map-like structure AWS uses (array of { Key: string, Value: string } where Key is unique
  const tags = [];
  if (input.domain.tags) {
    for (const [key, value] of Object.entries(input.domain.tags)) {
      tags.push({
        "Key": key,
        "Value": value,
      });
    }
    if (tags.length > 0) {
      object["TagSpecifications"] = [{
        "ResourceType": input.domain.awsResourceType,
        "Tags": tags
      }];
    }
  }

  return {
    format: "json",
    code: JSON.stringify(object, null, '\t'),
  };
}
