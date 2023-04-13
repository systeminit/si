async function poem(personName) {
  return {
    name: "si:poemWorkflow",
    kind: "conditional",
    steps: [
      //{
      //  workflow: "si:exceptionalWorkflow",
      //},
      {
        command: "si:leroLeroStanza1Command",
      },
      {
        command: "si:leroLeroStanza2Command",
      },
      {
        command: "si:leroLeroStanza3Command",
      },
      {
        command: "si:leroLeroStanza4Command",
      },
      {
        command: "si:leroLeroStanza5Command",
      },
      {
        command: "si:leroLeroStanza6Command",
      },
      {
        command: "si:leroLeroStanza7Command",
      },
      {
        workflow: "si:finalizingWorkflow",
        args: [personName],
      },
    ],
  };
}
