async function main() {
  console.log("Dummy Import Management Function");
  return {
    status: "ok",
    message: "Imported dummy asset",
    resource: {
      id: "dummy-imported",
      name: "imported-server",
      type: "Dummy::Server",
      properties: {
        size: "medium",
        region: "us-east-1",
      },
    },
  };
}