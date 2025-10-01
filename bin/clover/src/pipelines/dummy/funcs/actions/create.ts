async function main() {
  console.log("Dummy Create Action");
  return {
    status: "ok",
    message: "Dummy asset created successfully",
    payload: {
      id: `dummy-${Date.now()}`,
      status: "created",
    },
  };
}