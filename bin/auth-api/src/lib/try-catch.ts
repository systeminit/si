import * as _ from 'lodash';
import isPromise from 'is-promise';

// see https://www.npmjs.com/package/no-try for inspiration
// although their focus was not on the typing...

/** try-catch alternative that exposes a typed response rather than having it stuck in the try's scope */
export async function tryCatch<T>(
  tryFn: () => T | Promise<T>,
  catchFn: (Error) => void | Promise<void>,
): Promise<T> {
  try {
    return await tryFn();
  } catch (err) {
    const catchResult = catchFn(err);
    if (isPromise(catchResult)) { await catchResult; }
  }
}
