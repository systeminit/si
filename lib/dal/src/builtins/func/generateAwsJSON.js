function generateAwsJSON(component) {
  const ResourceType = component.properties.domain.awsResourceType
  const componentTags = component.properties.domain.tags ?? []

  const Tags = []

  for (const Key in componentTags) {
    Tags.push({
      Key,
      Value: componentTags[Key]
    })
  }

  delete component.properties.domain.tags
  delete component.properties.domain.awsResourceType
  delete component.properties.domain.region

  component.properties.domain.TagSpecifications = [{
    ResourceType, Tags
  }]

  return {
    format: "json",
    code: JSON.stringify(component.properties.domain, null, '\t')
  };
}
