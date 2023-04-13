async function generateAwsSecurityGroupJSON(input: Input): Promise<Output> {
  // Initialize the input JSON.
  const object = {
    "Description": input.domain.Description,
    "GroupName": input.domain.GroupName,
    "VpcId": input.domain.VpcId,
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
