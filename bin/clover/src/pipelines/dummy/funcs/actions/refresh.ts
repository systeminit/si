async function main() {
  console.log("Dummy Refresh Action");
  return {
    status: "ok",
    message: "Dummy asset refreshed successfully",
    payload: {
      id: "dummy-123",
      name: "test-server",
      status: "running",
      lastRefreshed: new Date().toISOString(),
    },
  };
}