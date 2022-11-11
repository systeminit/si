async function refresh(arg) {
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
