import { assertEquals, assertExists } from "@std/assert";
import { Context } from "../context.ts";
import { type TemplateComponent, TemplateContext } from "./context.ts";
import { initializeWorkingSet } from "./working_set.ts";

// Initialize Context once - this runs at module load time
await Context.init({ verbose: 0, noColor: true });

Deno.test("initializeWorkingSet() - creates deep copy from baseline", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set up baseline
  const baselineData: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "test-component",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: { "test": "value" },
    },
  ];
  ctx.baseline(baselineData);

  // Initialize working set
  initializeWorkingSet(ctx);

  // Verify working set was created
  const workingSet = ctx.workingSet();
  assertExists(workingSet);
  assertEquals(workingSet.length, 1);
  assertEquals(workingSet[0].id, "comp-1");
  assertEquals(workingSet[0].name, "test-component");
});

Deno.test("initializeWorkingSet() - handles undefined baseline gracefully", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // No baseline set
  assertEquals(ctx.baseline(), undefined);

  // Should not throw, just log warning
  initializeWorkingSet(ctx);

  // Working set should remain undefined
  assertEquals(ctx.workingSet(), undefined);
});

Deno.test("initializeWorkingSet() - handles empty baseline", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set empty baseline
  ctx.baseline([]);

  // Initialize working set
  initializeWorkingSet(ctx);

  // Working set should be empty array
  const workingSet = ctx.workingSet();
  assertExists(workingSet);
  assertEquals(workingSet.length, 0);
});

Deno.test("initializeWorkingSet() - working set is independent from baseline", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set up baseline
  const baselineData: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "original-name",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: {},
    },
  ];
  ctx.baseline(baselineData);

  // Initialize working set
  initializeWorkingSet(ctx);

  // Modify working set
  const workingSet = ctx.workingSet();
  assertExists(workingSet);
  workingSet[0].name = "modified-name";

  // Baseline should remain unchanged
  const baseline = ctx.baseline();
  assertExists(baseline);
  assertEquals(baseline[0].name, "original-name");
});

Deno.test("initializeWorkingSet() - creates deep copy of nested structures", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set up baseline with nested data
  const baselineData: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "test-component",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: { nested: { key: "value" } },
    },
  ];
  ctx.baseline(baselineData);

  // Initialize working set
  initializeWorkingSet(ctx);

  // Modify nested structure in working set
  const workingSet = ctx.workingSet();
  assertExists(workingSet);
  // deno-lint-ignore no-explicit-any
  (workingSet[0].attributes as any).nested.key = "modified";

  // Baseline should remain unchanged
  const baseline = ctx.baseline();
  assertExists(baseline);
  // deno-lint-ignore no-explicit-any
  assertEquals((baseline[0].attributes as any).nested.key, "value");
});

Deno.test("initializeWorkingSet() - handles multiple components", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set up baseline with multiple components
  const baselineData: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "component-1",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: {},
    },
    {
      id: "comp-2",
      name: "component-2",
      schemaId: "schema-2",
      resourceId: "res-2",
      attributes: {},
    },
    {
      id: "comp-3",
      name: "component-3",
      schemaId: "schema-3",
      resourceId: "res-3",
      attributes: {},
    },
  ];
  ctx.baseline(baselineData);

  // Initialize working set
  initializeWorkingSet(ctx);

  // Verify all components were copied
  const workingSet = ctx.workingSet();
  assertExists(workingSet);
  assertEquals(workingSet.length, 3);
  assertEquals(workingSet[0].id, "comp-1");
  assertEquals(workingSet[1].id, "comp-2");
  assertEquals(workingSet[2].id, "comp-3");
});
