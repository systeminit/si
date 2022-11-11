async function refresh(arg) {
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
