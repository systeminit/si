export interface ArgInput {
  toSet: string,
  readFrom: string,
}

export interface ArgOutput {
  toSet?: string,
  readFrom: string,
}

export interface RefreshOptions {
  inputs: Array<ArgInput>;
  outputs: Array<ArgOutput>;
}

export interface DeleteOptions {
  inputs: Array<ArgInput>;
}

export function parseInputOption(input: string): ArgInput {
    const [toSet, readFrom] = input.split(':');
    if (toSet && readFrom) {
    return { toSet, readFrom };
    } else {
      throw new Error(`Invalid input specifier; must be 'awsInputPath:siPropertiesPath': ${input}`);
    }
}

export function parseOutputOption(output: string): ArgOutput {
    if (output.includes(':')) {
      const [toSet, readFrom] = output.split(':');
      return { toSet, readFrom };
    } else {
      return { readFrom: output };
    }
}


export function makeRefreshOptions(options: { input: Array<string>, output: Array<string> }): RefreshOptions {
  const refreshOptions: RefreshOptions = { inputs: [], outputs: [] };
  for (const input of options.input) {
    const argInput = parseInputOption(input);
    refreshOptions.inputs.push(argInput);
  }
  for (const output of options.output) {
    const argOutput = parseOutputOption(output);
    refreshOptions.outputs.push(argOutput);
  }
  return refreshOptions;
}

export function makeDeleteOrActionOptions(options: { input: Array<string> }): DeleteOptions {
  const deleteOptions: RefreshOptions = { inputs: [], outputs: [] };
  for (const input of options.input) {
    const argInput = parseInputOption(input);
    deleteOptions.inputs.push(argInput);
  }
  return deleteOptions;
}
