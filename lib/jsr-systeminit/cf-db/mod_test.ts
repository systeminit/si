import { assertEquals, assertExists } from "@std/assert";
import { loadCfDatabase, getServiceByName, normalizeProperty } from "./mod.ts";

Deno.test("loadCfDatabase loads the database", async () => {
  const db = await loadCfDatabase({ services: ["AWS::Lambda::Function"] });
  assertExists(db);
  assertEquals(typeof db, "object");
});

Deno.test("getServiceByName returns a service", async () => {
  await loadCfDatabase({ services: ["AWS::Lambda::Function"] });
  const service = getServiceByName("AWS::Lambda::Function");
  assertExists(service);
  assertEquals(service.typeName, "AWS::Lambda::Function");
});

Deno.test("normalizeProperty handles string types", () => {
  const prop = {
    type: "string" as const,
    description: "A test string property",
  };
  const normalized = normalizeProperty(prop);
  assertEquals(normalized.type, "string");
  assertEquals(normalized.description, "A test string property");
});
