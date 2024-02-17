export interface RefreshInput {
  toSet: string,
  readFrom: string,
}

export interface RefreshOutput {
  toSet?: string,
  readFrom: string,
}

export interface RefreshOptions {
  inputs: Array<RefreshInput>;
  outputs: Array<RefreshOutput>;
}

export function makeRefreshOptions(options: { input: Array<string>, output: Array<string> }): RefreshOptions {
  const refreshOptions: RefreshOptions = { inputs: [], outputs: [] };
  for (const input of options.input) {
    const [toSet, readFrom] = input.split(':');
    if (toSet && readFrom) {
      refreshOptions.inputs.push({ toSet, readFrom });
    } else {
      throw new Error(`Invalid input specifier; must be 'awsInputPath:siPropertiesPath': ${input}`);
    }
  }
  for (const output of options.output) {
    if (output.includes(':')) {
      const [toSet, readFrom] = output.split(':');
      refreshOptions.outputs.push({ toSet, readFrom });
    } else {
      refreshOptions.outputs.push({ readFrom: output });
    }
  }
  return refreshOptions;
}
