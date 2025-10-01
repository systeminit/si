async function main() {
  console.log("Dummy Discover Management Function");
  return {
    status: "ok",
    message: "Discovered dummy assets",
    resources: [
      {
        id: "dummy-001",
        name: "server-1",
        type: "Dummy::Server",
      },
      {
        id: "dummy-002",
        name: "server-2",
        type: "Dummy::Server",
      },
    ],
  };
}