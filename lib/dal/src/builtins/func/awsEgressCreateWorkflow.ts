async function create(arg: Input): Promise<Output> {
  return {
    name: "si:awsEgressCreateWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsEgressCreateCommand",
        args: [arg],
      },
    ],
  };
}
