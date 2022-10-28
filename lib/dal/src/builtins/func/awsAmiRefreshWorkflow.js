async function refresh(arg) {
  return {
    name: "si:awsAmiRefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsAmiRefreshCommand",
        args: [arg],
      },
    ],
  };
}
