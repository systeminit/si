async function refresh(arg) {
  return {
    name: "si:awsKeyPairRefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsKeyPairRefreshCommand",
        args: [arg],
      },
    ],
  };
}
