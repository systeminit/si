async function create(arg: Input): Promise<Output> {
  return {
    name: "si:awsSecurityGroupCreateWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsSecurityGroupCreateCommand",
        args: [arg],
      },
    ],
  };
}
