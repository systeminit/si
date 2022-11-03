async function create(arg) {
  return {
    name: "si:awsEgressCreateWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsEgressCreateCommand",
        args: [arg],
      },
    ],
  };
}
