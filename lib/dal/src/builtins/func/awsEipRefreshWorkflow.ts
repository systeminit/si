async function refresh(arg: Input): Promise<Output> {
  return {
    name: "si:awsEipRefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsEipRefreshCommand",
        args: [arg],
      },
    ],
  };
}
