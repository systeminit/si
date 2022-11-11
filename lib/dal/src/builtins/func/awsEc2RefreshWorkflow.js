async function refresh(arg) {
  return {
    name: "si:awsEc2RefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:awsEc2RefreshCommand",
        args: [arg],
      },
    ],
  };
}
