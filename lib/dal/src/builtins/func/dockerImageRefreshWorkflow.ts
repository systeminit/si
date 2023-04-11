async function refresh(arg: Input): Promise<Output> {
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
