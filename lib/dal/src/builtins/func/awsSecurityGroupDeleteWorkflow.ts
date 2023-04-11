async function deleteResource(arg: Input): Promise<Output> {
  return {
    name: "si:awsSecurityGroupDeleteWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsSecurityGroupDeleteCommand",
        args: [arg],
      },
    ],
  };
}
