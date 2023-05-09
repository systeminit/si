async function deleteResource(arg: Input): Promise<Output> {
  return {
    name: "si:awsEgressDeleteWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsEgressDeleteCommand",
        args: [arg],
      },
    ],
  };
}
