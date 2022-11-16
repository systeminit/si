async function generateJSON(component) {
  // Initialize the input JSON.
  const object = {
    "KeyName": component.properties.domain.KeyName,
    "KeyType": component.properties.domain.KeyType,
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
