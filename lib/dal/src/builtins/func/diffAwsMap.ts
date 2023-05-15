async function diff(input: Input): Promise<Output> {
  if (Object.values(input.first).length !== input.second.length) {
    return true;
  }

  const keys = new Set(Object.keys(input.first));
  for (const row of input.second) {
    if (input.first[row.Key] !== row.Value) {
      return true;
    }
    keys.delete(row.Key);
  }
  
  return keys.size !== 0;
}
