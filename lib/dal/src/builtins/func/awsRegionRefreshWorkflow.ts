async function refresh(arg: Input): Promise<Output> {
  return {
    name: "si:awsRegionRefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsRegionRefreshCommand",
        args: [arg],
      },
    ],
  };
}
