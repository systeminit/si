async function refresh(arg) {
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
