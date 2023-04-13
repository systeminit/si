async function create(arg: Input): Promise<Output> {
  return {
    name: "si:awsEipCreateWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsEipCreateCommand",
        args: [arg],
      },
    ],
  };
}
