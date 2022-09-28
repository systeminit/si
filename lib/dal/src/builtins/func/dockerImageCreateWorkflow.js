async function create(arg) {
  return {
    name: "si:dockerImageCreateWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:dockerImageCreateCommand",
        args: [arg],
      },
    ],
  };
}
