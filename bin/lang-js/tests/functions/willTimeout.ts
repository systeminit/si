// deno-lint-ignore-file
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-nocheck
async function main() {
  console.log("yeehaw this is gonna explode!");
  // Just sleep for a very long time - timeout will kill it before this completes
  // eslint-disable-next-line no-promise-executor-return, arrow-parens
  await new Promise((r) => setTimeout(r, 60000));
  // eslint-disable-next-line no-autofix/no-unreachable
  console.log("this should be impossible to hit");
  return { status: "ok" };
}
