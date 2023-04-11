async function create(arg: Input): Promise<Output> {
  return {
    name: "si:awsKeyPairCreateWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsKeyPairCreateCommand",
        args: [arg],
      },
    ],
  };
}
