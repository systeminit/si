async function refresh(arg: Input): Promise<Output> {
  return {
    name: "si:awsIngressRefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsIngressRefreshCommand",
        args: [arg],
      },
    ],
  };
}
