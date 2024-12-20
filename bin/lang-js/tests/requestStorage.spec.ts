import { beforeAll, describe, expect, test } from "vitest";
import { makeBeforeRequestStorage } from "../src/sandbox/requestStorage";

describe("requestStorage", () => {
  const KEY = "a";
  const VALUE = "this is a test value";
  const EXEC_ID = "execId";

  beforeAll(() => {
    makeBeforeRequestStorage(EXEC_ID).setItem(KEY, VALUE);
  });

  test("Retrieve a value", () => {
    const value = makeBeforeRequestStorage(EXEC_ID).getItem(KEY);

    expect(value).toBe(VALUE);
  });

  test("Retrieve keys", () => {
    const keys = makeBeforeRequestStorage(EXEC_ID).getKeys();

    expect(keys).toHaveLength(1);
    expect(keys).toContain(KEY);
  });

  test("Delete a value by key", () => {
    makeBeforeRequestStorage(EXEC_ID).deleteItem(KEY);

    const value = makeBeforeRequestStorage(EXEC_ID).getItem(KEY);
    expect(value).toBeUndefined();
  });
});
