async function main(component: Input): Promise<Output> {
  const currentState = _.get(component, ["resource", "payload"]);
  const updateableProps: Array<Record<string, any>> =  _.get(component, ["domain", "Updateable"]);
  const desiredState = _.cloneDeep(currentState);
  _.merge(desiredState, updateableProps);
  const patch = jsonpatch.compare(currentState, desiredState, true);

  return {
    format: "json",
    code: JSON.stringify(patch, null, 2),
  };
}
