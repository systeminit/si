async function refresh(arg: Input): Promise<Output> {
  return {
    name: "si:awsSecurityGroupRefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsSecurityGroupRefreshCommand",
        args: [arg],
      },
    ],
  };
}
