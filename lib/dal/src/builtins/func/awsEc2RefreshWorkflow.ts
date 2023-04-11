async function refresh(arg: Input): Promise<Output> {
  return {
    name: "si:awsEc2RefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsEc2RefreshCommand",
        args: [arg],
      },
    ],
  };
}
