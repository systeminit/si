const { poem } = require("./workflow");
const { process } = require("./executor");

async function run() {
  await process(await poem("Paulo"));
}

run();
