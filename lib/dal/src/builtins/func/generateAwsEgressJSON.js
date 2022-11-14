async function generateJSON(component) {
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

  return {
    format: "json",
    code: JSON.stringify(object, null, '\t'),
  };
}
