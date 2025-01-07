import {
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.224.0/assert/mod.ts";
import { makeBeforeRequestStorage } from "../src/sandbox/requestStorage.ts";

const KEY = "a";
const VALUE = "this is a test value";
const EXEC_ID = "execId";

function setupStorage() {
  makeBeforeRequestStorage(EXEC_ID).setItem(KEY, VALUE);
}

Deno.test("requestStorage", async (t) => {
  await t.step("Retrieve a value", () => {
    setupStorage();
    const value = makeBeforeRequestStorage(EXEC_ID).getItem(KEY);
    assertEquals(value, VALUE);
  });

  await t.step("Retrieve keys", () => {
    setupStorage();
    const keys = makeBeforeRequestStorage(EXEC_ID).getKeys();
    assertEquals(keys.length, 1);
    assertEquals(keys.includes(KEY), true);
  });

  await t.step("Delete a value by key", () => {
    setupStorage();
    makeBeforeRequestStorage(EXEC_ID).deleteItem(KEY);
    const value = makeBeforeRequestStorage(EXEC_ID).getItem(KEY);
    assertExists(!value);
  });
});
