async function main(component: Input): Promise<Output> {
  const resourceBody = component.domain?.CloudFormationResourceBody;

  if (!resourceBody) {
    return {
      format: "json",
      code: JSON.stringify(
        {
          AWSTemplateFormatVersion: "2010-09-09",
          Resources: {},
        },
        null,
        2,
      ),
    };
  }

  // CloudFormationResourceBody is already a JSON string from the attribute function
  const resource = JSON.parse(resourceBody);

  const cloudFormationTemplate = {
    AWSTemplateFormatVersion: "2010-09-09",
    Resources: {
      cfnResource: resource,
    },
  };

  return {
    format: "json",
    code: JSON.stringify(cloudFormationTemplate, null, 2),
  };
}
