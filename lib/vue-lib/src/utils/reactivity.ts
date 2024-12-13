import { reactive } from "vue";

export function usePromise<T>(promise: PromiseLike<T>): ReactivePromise<T>;
export function usePromise(promise: undefined): undefined;
export function usePromise<T>(
  promise?: PromiseLike<T>,
): ReactivePromise<T> | undefined;
export function usePromise<T>(promise?: PromiseLike<T>) {
  if (!promise) return undefined;
  if (promise instanceof ReactivePromise) return promise;
  return new ReactivePromise(promise);
}

export class ReactivePromise<T = unknown> implements PromiseLike<T> {
  value?: T;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  error?: any;

  get state() {
    if (!this.promise) return "pending";
    return "error" in this ? "rejected" : "fulfilled";
  }
  get isPending() {
    return "promise" in this;
  }
  get isSuccess() {
    return "value" in this;
  }
  get isError() {
    return "error" in this;
  }

  /** Return a new reactive promise with the new value */
  then<V = T, E = never>(
    onfulfilled?: ((value: T) => V | PromiseLike<V>) | null,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onrejected?: ((reason: any) => E | PromiseLike<E>) | null,
  ) {
    const promise: PromiseLike<T> =
      this.promise ??
      (this.isError
        ? Promise.reject<T>(this.error)
        : Promise.resolve(this.value as T));
    return usePromise(promise.then(onfulfilled, onrejected));
  }

  private promise?: PromiseLike<T>;
  constructor(promise: PromiseLike<T>) {
    this.promise = promise.then(
      (value) => {
        delete this.promise;
        this.value = value;
        return this.value;
      },
      (error) => {
        delete this.promise;
        this.error = error;
        return this.error;
      },
    );
    // Make ourselves reactive before we return!
    // eslint-disable-next-line no-constructor-return
    return reactive(this) as ReactivePromise<T>;
  }
}
