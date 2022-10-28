async function refresh(arg) {
  return {
    name: "si:dockerImageRefreshWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:dockerImageRefreshCommand",
        args: [arg],
      },
    ],
  };
}
