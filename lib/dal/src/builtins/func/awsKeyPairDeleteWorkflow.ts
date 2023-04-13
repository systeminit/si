async function deleteResource(arg: Input): Promise<Output> {
  return {
    name: "si:awsKeyPairDeleteWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsKeyPairDeleteCommand",
        args: [arg],
      },
    ],
  };
}
