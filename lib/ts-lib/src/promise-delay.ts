export function promiseDelay(delayInMs: number) {
  new Promise((resolve) => {
    setTimeout(resolve, delayInMs);
  });
}