async function exceptional() {
  return {
    name: "si:exceptional",
    kind: "exceptional",
    steps: [
      {
        command: "si:title",
      },
      {
        command: "si:title2",
      },
    ],
  };
};
