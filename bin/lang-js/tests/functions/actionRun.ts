// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-nocheck
async function workit() {
  console.log('first');
  console.log('second');
  // eslint-disable-next-line no-promise-executor-return
  const sleep = new Promise((resolve) => setTimeout(resolve, 15));
  await sleep;
  return { status: 'ok' };
}
