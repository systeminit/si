async function main() {
  console.log("Dummy Update Action");
  return {
    status: "ok",
    message: "Dummy asset updated successfully",
    payload: {
      status: "updated",
    },
  };
}