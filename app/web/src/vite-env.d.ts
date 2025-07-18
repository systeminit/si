declare const __COMMIT_HASH__: string;
declare const __SHARED_WORKER_HASH__: string;
declare const __WEBWORKER_HASH__: string;

declare module "@sqlite.org/sqlite-wasm" {
  interface SAHPoolUtil {
    isPaused(): boolean;
    pauseVfs(): SAHPoolUtil;
    unpauseVfs(): Promise<void>;
  }
}
