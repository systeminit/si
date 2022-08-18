async function poem(personName) {
  return {
    name: "si:poem",
    kind: "conditional",
    steps: [
      //{
      //  workflow: "si:exceptional",
      //},
      {
        command: "si:firstStanza",
      },
      {
        command: "si:secondStanza",
      },
      {
        command: "si:thirdStanza",
      },
      {
        command: "si:fourthStanza",
      },
      {
        command: "si:fifthStanza",
      },
      {
        command: "si:sixthStanza",
      },
      {
        command: "si:seventhStanza",
      },
      {
        workflow: "si:finalizing",
        args: [personName],
      },
    ],
  };
}
