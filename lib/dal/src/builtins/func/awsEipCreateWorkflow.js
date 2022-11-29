async function create(arg) {
  return {
    name: "si:awsEipCreateWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsEipCreateCommand",
        args: [arg],
      },
    ],
  };
}
