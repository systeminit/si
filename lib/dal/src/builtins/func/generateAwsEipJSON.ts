async function generateAwsEipJSON(input: Input): Promise<Output> {
  // Initialize the input JSON.
  const object = {
    Domain: "vpc",
  };

  const tags = [];
  if (input.domain.tags) {
    for (const [key, value] of Object.entries(input.domain.tags)) {
      tags.push({
        Key: key,
        Value: value,
      });
    }
    if (tags.length > 0) {
      object["TagSpecifications"] = [
        {
          ResourceType: input.domain.awsResourceType,
          Tags: tags,
        },
      ];
    }
  }

  return {
    format: "json",
    code: JSON.stringify(object, null, "\t"),
  };
}
