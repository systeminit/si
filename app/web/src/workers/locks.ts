import { SqlValue } from "@sqlite.org/sqlite-wasm";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyFn = (...args: any[]) => any;

export class ReadWriteLock {
  name: string;
  readerCount: number;
  writeLockAcquired: boolean;

  constructor(name: string) {
    this.name = name;
    this.readerCount = 0;
    this.writeLockAcquired = false;
  }

  isWriteLockAcquired(): boolean {
    return this.writeLockAcquired;
  }

  async readLock(callback: AnyFn): Promise<SqlValue[][]> {
    return await navigator.locks.request(`${this.name}-reader`, { mode: "shared" }, async () => {
      this.readerCount++;
      try {
        return await callback();
      } finally {
        this.readerCount--;
      }
    });
  }

  async writeLock(callback: AnyFn): Promise<SqlValue[][]> {
    try {
      return await navigator.locks.request(`${this.name}-reader`, { mode: "exclusive" }, async () => {
        return await navigator.locks.request(`${this.name}-writer`, { mode: "exclusive" }, async () => {
          this.writeLockAcquired = true;
          return await callback();
        });
      });
    } finally {
      this.writeLockAcquired = false;
    }
  }

  async query() {
    const state = await navigator.locks.query();
    const readerLocks = state.held?.filter((lock) => lock.name === `${this.name}-reader`);
    const writerLocks = state.held?.filter((lock) => lock.name === `${this.name}-writer`);

    return {
      readers: readerLocks?.length ?? 0,
      writers: writerLocks?.length ?? 0,
      pending:
        state.pending?.filter((lock) => lock.name === `${this.name}-reader` || lock.name === `${this.name}-writer`)
          .length ?? 0,
    };
  }
}
