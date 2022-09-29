function generateAwsJSON(component) {
  const ResourceType = component.properties.domain.awsResourceType

  if (ResourceType !== undefined) {
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

    component.properties.domain.TagSpecifications = [{
      ResourceType, Tags
    }]
  }

  delete component.properties.domain.region

  return {
    format: "json",
    code: JSON.stringify(component.properties.domain, null, '\t')
  };
}
