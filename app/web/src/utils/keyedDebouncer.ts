import { debounce, DebouncedFunc } from "lodash-es";

export default <F extends (...args: Parameters<F>) => ReturnType<F>>(toDebounce: F, waitMs?: number) => {
  const debounceQueues: {
    [key: string | number | symbol]: DebouncedFunc<F> | undefined;
  } = {};
  return (key: string | number | symbol) => {
    if (!debounceQueues[key]) {
      debounceQueues[key] = debounce(toDebounce, waitMs ?? 1000);
    }
    return debounceQueues[key];
  };
};
