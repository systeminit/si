// deno-lint-ignore-file
// @ts-nocheck
function main() {
  console.log("Running Main");
  const b1 = requestStorage.getEnv("b1");
  const b2 = requestStorage.getEnv("b2");
  const b3 = requestStorage.getEnv("b3");

  const keys = requestStorage.getEnvKeys();

  console.log(
    `Before function 1 set b1="${b1}", Before function 2 said "${b3}", keys are ${keys}`,
  );

  return {
    status: b1 && b2 === undefined && typeof b3 === "string" ? "ok" : "error",
  };
}
