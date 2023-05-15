async function diff(input: Input): Promise<Output> {
  const normalizedResponse = {
    diff: true,
    newValue: input.second.reduce((acc, { Key, Value }) => ({ ...acc, [Key]: Value }), {}),
  };

  if (Object.values(input.first).length !== input.second.length) {
    return normalizedResponse;
  }

  const keys = new Set(Object.keys(input.first));
  for (const row of input.second) {
    if (input.first[row.Key] !== row.Value) {
      return normalizedResponse;
    }
    keys.delete(row.Key);
  }
  
  if (keys.size !== 0) {
    return normalizedResponse;
  } else {
    return { diff: false };
  }
}
