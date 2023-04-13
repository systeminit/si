async function deleteResource(arg: Input): Promise<Output> {
  return {
    name: "si:awsIngressDeleteWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsIngressDeleteCommand",
        args: [arg],
      },
    ],
  };
}
