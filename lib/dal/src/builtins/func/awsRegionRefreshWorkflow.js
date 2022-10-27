async function refresh(arg) {
  return {
    name: "si:awsRegionRefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsRegionRefreshCommand",
        args: [arg],
      },
    ],
  };
}
