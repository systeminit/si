async function main(input: Input): Promise<Output> {
  const properties = _.cloneDeep(input.cfnProperties);
  delete properties["extra"];
  const result = {
    "Type": input.cfnType,
    "Properties": properties,
  };
  return result;
}
