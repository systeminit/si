async function whatIsLove() {
  return {
    name: "si:whatIsLoveWorkflow",
    kind: "conditional",
    steps: [
      {
        command: "si:babyDontHurtMeCommand",
      },
    ],
  };
}
