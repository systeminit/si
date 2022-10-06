const promiseDelay = (delayInMs: number) =>
  new Promise((resolve) => {
    setTimeout(resolve, delayInMs);
  });

export default promiseDelay;
