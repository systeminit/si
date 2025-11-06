import { assertEquals } from "@std/assert";
import { Context } from "../context.ts";
import { TemplateComponent, TemplateContext } from "./context.ts";
import { applyTransform } from "./transform.ts";

// Initialize Context once - this runs at module load time
await Context.init({ verbose: 0, noColor: true });

Deno.test("applyTransform() - applies simple transform function", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [
    {
      id: "comp-1",
      schemaId: "schema-1",
      name: "server-01",
      resourceId: "res-1",
      attributes: {},
    },
    {
      id: "comp-2",
      schemaId: "schema-1",
      name: "server-02",
      resourceId: "res-2",
      attributes: {},
    },
  ];
  tctx.workingSet(workingSet);

  // Set transform function that adds a prefix to all names
  tctx.transform((components) => {
    return components.map((c) => ({
      ...c,
      name: `transformed-${c.name}`,
    }));
  });

  await applyTransform(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "transformed-server-01");
  assertEquals(result![1].name, "transformed-server-02");
});

Deno.test("applyTransform() - transform function receives input data", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up input data
  tctx.inputData({ prefix: "prod" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "server",
    resourceId: "res-1",
    attributes: {},
  }];
  tctx.workingSet(workingSet);

  // Set transform function that uses input data
  tctx.transform((components, inputData) => {
    const prefix = (inputData as { prefix?: string })?.prefix || "default";
    return components.map((c) => ({
      ...c,
      name: `${prefix}-${c.name}`,
    }));
  });

  await applyTransform(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "prod-server");
});

Deno.test("applyTransform() - no-op when transform is undefined", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "server-01",
    resourceId: "res-1",
    attributes: {},
  }];
  tctx.workingSet(workingSet);

  // Don't set transform function - should be no-op
  await applyTransform(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "server-01");
});

Deno.test("applyTransform() - no-op when workingSet is undefined", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set transform function but don't set working set
  tctx.transform((components) => {
    return components.map((c) => ({
      ...c,
      name: `transformed-${c.name}`,
    }));
  });

  // Should not throw - just no-op
  await applyTransform(tctx);

  assertEquals(tctx.workingSet(), undefined);
});

Deno.test("applyTransform() - transform can filter components", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [
    {
      id: "comp-1",
      schemaId: "schema-1",
      name: "prod-server-01",
      resourceId: "res-1",
      attributes: {},
    },
    {
      id: "comp-2",
      schemaId: "schema-1",
      name: "dev-server-01",
      resourceId: "res-2",
      attributes: {},
    },
    {
      id: "comp-3",
      schemaId: "schema-1",
      name: "prod-server-02",
      resourceId: "res-3",
      attributes: {},
    },
  ];
  tctx.workingSet(workingSet);

  // Set transform function that filters to only prod components
  tctx.transform((components) => {
    return components.filter((c) => c.name.startsWith("prod-"));
  });

  await applyTransform(tctx);

  const result = tctx.workingSet();
  assertEquals(result!.length, 2);
  assertEquals(result![0].name, "prod-server-01");
  assertEquals(result![1].name, "prod-server-02");
});

Deno.test("applyTransform() - transform can modify component attributes", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "server",
    resourceId: "res-1",
    attributes: {
      "/si/name": "server",
      "/domain/port": 8080,
    },
  }];
  tctx.workingSet(workingSet);

  // Set transform function that modifies attributes
  tctx.transform((components) => {
    return components.map((c) => ({
      ...c,
      attributes: {
        ...c.attributes,
        "/domain/port": 9090,
        "/domain/newField": "added",
      },
    }));
  });

  await applyTransform(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].attributes["/domain/port"], 9090);
  assertEquals(result![0].attributes["/domain/newField"], "added");
});

Deno.test("applyTransform() - transform with multiple components", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set with multiple components
  const workingSet: TemplateComponent[] = [
    {
      id: "comp-1",
      schemaId: "schema-1",
      name: "server-01",
      resourceId: "res-1",
      attributes: { "/domain/index": 1 },
    },
    {
      id: "comp-2",
      schemaId: "schema-1",
      name: "server-02",
      resourceId: "res-2",
      attributes: { "/domain/index": 2 },
    },
    {
      id: "comp-3",
      schemaId: "schema-1",
      name: "server-03",
      resourceId: "res-3",
      attributes: { "/domain/index": 3 },
    },
  ];
  tctx.workingSet(workingSet);

  // Set transform function that processes all components
  tctx.transform((components) => {
    return components.map((c) => ({
      ...c,
      name: `processed-${c.name}`,
      attributes: {
        ...c.attributes,
        "/domain/index": (c.attributes["/domain/index"] as number) * 10,
      },
    }));
  });

  await applyTransform(tctx);

  const result = tctx.workingSet();
  assertEquals(result!.length, 3);
  assertEquals(result![0].name, "processed-server-01");
  assertEquals(result![0].attributes["/domain/index"], 10);
  assertEquals(result![1].name, "processed-server-02");
  assertEquals(result![1].attributes["/domain/index"], 20);
  assertEquals(result![2].name, "processed-server-03");
  assertEquals(result![2].attributes["/domain/index"], 30);
});

Deno.test("applyTransform() - transform can return empty array", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [
    {
      id: "comp-1",
      schemaId: "schema-1",
      name: "server-01",
      resourceId: "res-1",
      attributes: {},
    },
    {
      id: "comp-2",
      schemaId: "schema-1",
      name: "server-02",
      resourceId: "res-2",
      attributes: {},
    },
  ];
  tctx.workingSet(workingSet);

  // Set transform function that filters out all components
  tctx.transform((components) => {
    return components.filter((c) => c.name === "nonexistent");
  });

  await applyTransform(tctx);

  const result = tctx.workingSet();
  assertEquals(result!.length, 0);
});

Deno.test("applyTransform() - transform with undefined input data", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Don't set input data

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "server",
    resourceId: "res-1",
    attributes: {},
  }];
  tctx.workingSet(workingSet);

  // Set transform function that handles undefined input data
  tctx.transform((components, inputData) => {
    const prefix = (inputData as { prefix?: string })?.prefix || "default";
    return components.map((c) => ({
      ...c,
      name: `${prefix}-${c.name}`,
    }));
  });

  await applyTransform(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "default-server");
});

Deno.test("applyTransform() - transform can add new components", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "server",
    resourceId: "res-1",
    attributes: {},
  }];
  tctx.workingSet(workingSet);

  // Set transform function that duplicates components
  tctx.transform((components) => {
    const duplicated: TemplateComponent[] = [];
    for (const c of components) {
      duplicated.push(c);
      duplicated.push({
        ...c,
        id: `${c.id}-copy`,
        name: `${c.name}-copy`,
        resourceId: `${c.resourceId}-copy`,
      });
    }
    return duplicated;
  });

  await applyTransform(tctx);

  const result = tctx.workingSet();
  assertEquals(result!.length, 2);
  assertEquals(result![0].name, "server");
  assertEquals(result![1].name, "server-copy");
});
