import { ref, Ref, watchEffect, WatchEffectOptions } from "vue";

/**
 * Computed value that is set by an async function.
 *
 * Debounced: async function will not be run more than once at a time; if dependencies change
 * while the async function is running, it will run again after completion.
 */
export function computedAsyncDebounce<T>(
  getter: () => Promise<T>,
  initialValue: T,
): Ref<T>;
export function computedAsyncDebounce<T>(
  getter: () => Promise<T>,
  initialValue?: T,
): Ref<T | undefined>;
export function computedAsyncDebounce<T>(
  getter: () => Promise<T>,
  initialValue?: T,
) {
  const result = ref<T>();
  result.value = initialValue;
  watchEffectAsyncDebounce(async () => {
    result.value = await getter();
  });
  return result;
}

/**
 * Run an async function whenever its dependencies change.
 *
 * Debounced: async function will not be run more than once at a time; if dependencies change
 * while the async function is running, it will run again after completion.
 */
export function watchEffectAsyncDebounce(
  f: () => Promise<void>,
  options?: WatchEffectOptions,
) {
  const hasPause = ref(false);
  const watchHandle = watchEffect(async () => {
    // Wait until we've got pause and resume set before we start actually running (i.e. don't
    // do it until watchEffect has returned for the first time and handed us handlers).
    if (!hasPause.value) return;

    // Pause the watch from running again while we await the promise.
    watchHandle.pause();
    try {
      await f();
    } finally {
      watchHandle.resume();
    }
  }, options);
  hasPause.value = true;
  return watchHandle;
}
