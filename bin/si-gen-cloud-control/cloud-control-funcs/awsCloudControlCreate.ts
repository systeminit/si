async function main(component: Input): Promise<Output> {
  interface CloudControlPayload {
    TypeName: string;
    DesiredState: Record<string, any>;
  }

  const cloudControlType = _.get(component, [
    "domain",
    "extra",
    "AwsResourceType",
  ]);

  const createOnlyProps: Array<Record<string, any>> = _.get(component, [
    "domain",
    "Create Only",
  ]);
  const updateableProps: Array<Record<string, any>> = _.get(component, [
    "domain",
    "Updateable",
  ]);
  const cloudControlProperties: Record<string, any> = {};
  _.merge(cloudControlProperties, createOnlyProps, updateableProps);

  const cloudControlPayload: CloudControlPayload = {
    TypeName: cloudControlType,
    DesiredState: cloudControlProperties,
  };

  return {
    format: "json",
    code: JSON.stringify(cloudControlPayload, null, 2),
  };
}
