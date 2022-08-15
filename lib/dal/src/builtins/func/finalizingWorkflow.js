async function finalizing(personName, lastReturn) {
  console.log(`Finalizing workflow, lastReturn: ${lastReturn}`);
  return {
    name: "si:finalizing",
    kind: "parallel",
    steps: [
      {
        command: "si:question",
        args: [personName],
      },
      {
        command: "si:bye",
      },
    ],
  };
};
