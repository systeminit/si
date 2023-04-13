async function failure() {
  return {
    name: "si:failureWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:failCommand",
      },
    ],
  };
}
