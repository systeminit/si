import * as _ from "lodash-es";
import type { DebouncedFunc } from "lodash-es";

/* eslint-disable @typescript-eslint/no-explicit-any */
type AnyFn = (...args: any[]) => any;

export interface MemoizeDebouncedFn<F extends AnyFn> extends DebouncedFunc<F> {
  (...args: Parameters<F>): ReturnType<F> | undefined;
  flush: (...args: Parameters<F>) => ReturnType<F> | undefined;
  cancel: (...args: Parameters<F>) => void;
}
/**
 * Debounce based on args to the fn
 */
export function memoizeDebounce<F extends AnyFn>(
  func: F,
  wait = 0,
  options: _.DebounceSettings = {},
  resolver?: (...args: Parameters<F>) => unknown,
): MemoizeDebouncedFn<F> {
  const dbMemo = _.memoize<(...args: Parameters<F>) => _.DebouncedFunc<F>>(
    (..._args: Parameters<F>) => _.debounce(func, wait, options),
    resolver,
  );

  function wrappedFn(this: MemoizeDebouncedFn<F>, ...args: Parameters<F>): ReturnType<F> | undefined {
    return dbMemo(...args)(...args);
  }

  const flush: MemoizeDebouncedFn<F>["flush"] = (...args) => {
    return dbMemo(...args).flush();
  };

  const cancel: MemoizeDebouncedFn<F>["cancel"] = (...args) => {
    return dbMemo(...args).cancel();
  };

  wrappedFn.flush = flush;
  wrappedFn.cancel = cancel;

  return wrappedFn;
}

/**
 * Throttle based on args to the fn
 */
export function memoizeThrottle<F extends AnyFn>(
  func: F,
  wait = 0,
  options: _.ThrottleSettings = {},
  resolver?: (...args: Parameters<F>) => unknown,
): MemoizeDebouncedFn<F> {
  // const memoized = _.memoize((...args: Parameters<F>) => {
  //   // The memoized key is used to create a new throttled instance
  //   const throttled = _.throttle(func, wait, options);
  //   return throttled(...args);
  // }, resolver); // Use the resolver to define the cache key

  // return memoized;

  const throttleMemo = _.memoize<(...args: Parameters<F>) => _.DebouncedFunc<F>>(
    (..._args: Parameters<F>) => _.throttle(func, wait, options),
    resolver,
  );

  function wrappedFn(this: MemoizeDebouncedFn<F>, ...args: Parameters<F>): ReturnType<F> | undefined {
    return throttleMemo(...args)(...args);
  }

  const flush: MemoizeDebouncedFn<F>["flush"] = (...args) => {
    return throttleMemo(...args).flush();
  };

  const cancel: MemoizeDebouncedFn<F>["cancel"] = (...args) => {
    return throttleMemo(...args).cancel();
  };

  wrappedFn.flush = flush;
  wrappedFn.cancel = cancel;

  return wrappedFn;
}
