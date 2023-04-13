async function create(arg: Input): Promise<Output> {
  return {
    name: "si:awsEc2CreateWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsEc2CreateCommand",
        args: [arg],
      },
    ],
  };
}
