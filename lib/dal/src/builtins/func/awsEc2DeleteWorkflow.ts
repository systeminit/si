async function deleteResource(arg: Input): Promise<Output> {
  return {
    name: "si:awsEc2DeleteWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsEc2DeleteCommand",
        args: [arg],
      },
    ],
  };
}
