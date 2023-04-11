async function finalizing(personName, lastReturn) {
  console.log(`Finalizing workflow, lastReturn: ${lastReturn}`);
  return {
    name: "si:finalizingWorkflow",
    kind: "parallel",
    steps: [
      {
        command: "si:leroLeroQuestionCommand",
        args: [personName],
      },
      {
        command: "si:leroLeroByeCommand",
      },
    ],
  };
}
