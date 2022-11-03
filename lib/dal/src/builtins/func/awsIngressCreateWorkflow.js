async function create(arg) {
  return {
    name: "si:awsIngressCreateWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsIngressCreateCommand",
        args: [arg],
      },
    ],
  };
}
