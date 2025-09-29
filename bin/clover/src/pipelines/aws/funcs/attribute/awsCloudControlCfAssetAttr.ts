async function main(input: Input): Promise<Output> {
  const properties = _.cloneDeep(input.cfnProperties);
  delete properties["extra"];
  const result = {
    "LogicalResourceName": input.cfnLogicalResourceName,
    "Type": input.cfnType,
    "Properties": properties,
  };
  return result;
}
