async function generateAwsIngressJSON(input) {
  // Initialize the input JSON.
  const object = {
    "GroupId": input.domain.GroupId,
    "IpPermissions": [],
  };

  if (input.domain.IpPermissions) {
    for (const value of input.domain.IpPermissions) {
      object["IpPermissions"].push({
        "FromPort": parseInt(value.FromPort),
        "ToPort": parseInt(value.ToPort),
        "IpProtocol": value.IpProtocol,
        "IpRanges": [{ "CidrIp": value.CidrIp }],
      });
    }
  }

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
