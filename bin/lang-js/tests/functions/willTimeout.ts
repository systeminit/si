// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-nocheck
async function main() {
  console.log("yeehaw this is gonna explode!");
  // eslint-disable-next-line no-constant-condition
  while (true) {
    // eslint-disable-next-line no-promise-executor-return, arrow-parens
    await new Promise((r) => setTimeout(r, 1000));
  }
  // eslint-disable-next-line no-autofix/no-unreachable
  console.log("this should be impossible to hit");
  return { status: "ok" };
}
