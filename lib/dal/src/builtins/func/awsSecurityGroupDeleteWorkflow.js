async function deleteResource(arg) {
  return {
    name: "si:awsSecurityGroupDeleteWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsSecurityGroupDeleteCommand",
        args: [arg],
      },
    ],
  };
}
