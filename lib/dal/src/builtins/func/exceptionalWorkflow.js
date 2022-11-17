async function exceptional() {
  return {
    name: "si:exceptionalWorkflow",
    kind: "exceptional",
    steps: [
      {
        command: "si:leroLeroTitle1Command",
      },
      {
        command: "si:leroLeroTitle2Command",
      },
    ],
  };
}
