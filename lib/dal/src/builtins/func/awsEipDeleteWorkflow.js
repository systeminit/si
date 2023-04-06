async function deleteResource(arg) {
  return {
    name: "si:awsEipDeleteWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsEipDeleteCommand",
        args: [arg],
      },
    ],
  };
}
