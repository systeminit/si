async function create(arg) {
  return {
    name: "si:awsKeyPairCreateWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsKeyPairCreateCommand",
        args: [arg],
      },
    ],
  };
}
