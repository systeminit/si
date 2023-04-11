async function create(arg: Input): Promise<Output> {
  return {
    name: "si:awsIngressCreateWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsIngressCreateCommand",
        args: [arg],
      },
    ],
  };
}
