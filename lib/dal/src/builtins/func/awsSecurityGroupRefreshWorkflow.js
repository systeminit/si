async function refresh(arg) {
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
