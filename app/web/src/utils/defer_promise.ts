// useful when you need to export a promise before ready to start whatever will initialize it
// should be used sparingly
export default function defer<T = unknown>() {
  // set to noop, but they will be replaced immediately
  let resolve: (value?: T) => void = () => {};
  let reject: (reason?: unknown) => void = () => {};
  const promise = new Promise((_resolve, _reject) => {
    resolve = _resolve;
    reject = _reject;
  });

  return { promise, resolve, reject };
}

export type DeferredPromise<T> = ReturnType<typeof defer<T>>;
