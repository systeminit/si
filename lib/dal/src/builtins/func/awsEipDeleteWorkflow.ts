async function deleteResource(arg: Input): Promise<Output> {
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
