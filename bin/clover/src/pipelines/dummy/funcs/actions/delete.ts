async function main() {
  console.log("Dummy Delete Action");
  return {
    status: "ok",
    message: "Dummy asset deleted successfully",
    payload: {
      status: "deleted",
    },
  };
}