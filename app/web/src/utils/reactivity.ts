import { Reactive, reactive } from "vue";

export function usePromise<T>(
  promise: PromiseLike<T>,
): Reactive<PromiseState<T> & PromiseLike<T>>;
export function usePromise(promise: undefined): undefined;
export function usePromise<T>(
  promise: PromiseLike<T> | undefined,
): Reactive<PromiseState<T> & PromiseLike<T>> | undefined;
export function usePromise<T>(promise?: PromiseLike<T> | undefined) {
  if (!promise) return undefined;

  // When the promise completes, update the result state
  // (because these are functions they can access state, even though it's declared later)
  const setState = promise.then(
    (value) => {
      Object.assign(state, { state: "success", value });
      return value;
    },
    (error) => {
      Object.assign(state, { state: "error", error });
      throw error;
    },
  );

  // Start with the pending state
  const state = Object.assign(setState, {
    state: "pending",
  } as PromiseState<T>);

  // Make it reactive for the caller
  return reactive(state);
}

export type UsePromise<T = unknown> = ReturnType<typeof usePromise<T>>;

export type PromiseState<T = unknown> = Readonly<
  | {
      state: "pending";
      isPending: true;
      isSuccess?: undefined;
      isError?: undefined;
      value?: undefined;
      error?: undefined;
    }
  | {
      state: "success";
      value: T;
      isPending: false;
      isSuccess: true;
      isError: false;
      error?: undefined;
    }
  | {
      state: "error";
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      error: any;
      isPending: false;
      isSuccess: false;
      isError: true;
      value?: undefined;
    }
>;
