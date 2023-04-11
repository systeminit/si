async function refresh(arg: Input): Promise<Output> {
  return {
    name: "si:awsAmiRefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsAmiRefreshCommand",
        args: [arg],
      },
    ],
  };
}
