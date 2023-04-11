async function refresh(arg: Input): Promise<Output> {
  return {
    name: "si:awsEgressRefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsEgressRefreshCommand",
        args: [arg],
      },
    ],
  };
}
