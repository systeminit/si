exports.finalizing = async (personName, lastReturn) => {
  console.log(`Finalizing workflow, lastReturn: ${lastReturn}`);
  return {
    name: "finalizing",
    kind: "parallel",
    steps: [
      {
        command: "question",
        args: [personName],
      },
      {
        command: "bye",
      },
    ],
  };
};

exports.exceptional = async () => {
  return {
    name: "exceptional",
    kind: "exception",
    steps: [
      {
        command: "title",
      },
      {
        command: "title2",
      },
    ],
  };
};

exports.poem = async (personName) => {
  return {
    name: "poem",
    kind: "conditional",
    steps: [
      {
        workflow: "exceptional",
      },
      {
        command: "firstStanza",
      },
      {
        command: "secondStanza",
      },
      {
        command: "thirdStanza",
      },
      {
        command: "fourthStanza",
      },
      {
        command: "fifthStanza",
      },
      {
        command: "sixthStanza",
      },
      {
        command: "seventhStanza",
      },
      {
        workflow: "finalizing",
        args: [personName],
      },
    ],
  };
}
