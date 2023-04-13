async function refresh(arg: Input): Promise<Output> {
  return {
    name: "si:awsKeyPairRefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsKeyPairRefreshCommand",
        args: [arg],
      },
    ],
  };
}
