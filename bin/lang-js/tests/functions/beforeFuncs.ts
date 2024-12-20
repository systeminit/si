// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-nocheck
function main() {
  console.log("Running Main");
  const b1 = requestStorage.getItem("b1");
  const b2 = requestStorage.getItem("b2");
  const b3 = requestStorage.getItem("b3");

  const keys = requestStorage.getKeys();

  console.log(
    `Before function 1 set b1="${b1}", Before function 2 said "${b3}", keys are ${keys}`,
  );

  return {
    status: b1 && b2 === undefined && typeof b3 === "string" ? "ok" : "error",
  };
}

function before1(arg) {
  console.log("Running Before 1");
  console.log(`My arg is ${arg}`);
  requestStorage.setItem("b1", true);
  requestStorage.setItem("b2", true);
}

function before2(arg) {
  console.log("Running Before 2");
  console.log(`My arg is ${arg}`);
  requestStorage.deleteItem("b2");
  requestStorage.setItem("b3", "I'm a string");
}
