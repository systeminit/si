async function create(arg) {
  return {
    name: "si:awsSecurityGroupCreateWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsSecurityGroupCreateCommand",
        args: [arg],
      },
    ],
  };
}
