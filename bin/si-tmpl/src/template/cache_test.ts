import { assertEquals } from "@std/assert";
import { parse as parseYaml } from "@std/yaml";
import { Context } from "../context.ts";
import { TemplateComponent, TemplateContext } from "./context.ts";
import { BaselineCache, cacheBaseline } from "./cache.ts";

// Initialize Context once - this runs at module load time
await Context.init({ verbose: 0, noColor: true });

Deno.test("cacheBaseline() - writes JSON format correctly", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set up baseline and schema cache
  const components: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "test-component",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: {},
    },
  ];
  ctx.baseline(components);

  const schemaCache = ctx.schemaCache();
  schemaCache.set("schema-1", {
    schemaId: "schema-1",
    name: "TestSchema",
    defaultVariantId: "variant-1",
    variantIds: ["variant-1"],
  });

  // Cache to temp file
  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  try {
    await cacheBaseline(ctx, tempFile);

    // Verify file was written
    const content = await Deno.readTextFile(tempFile);
    const data = JSON.parse(content) as BaselineCache;

    assertEquals(data.components.length, 1);
    assertEquals(data.components[0].id, "comp-1");
    assertEquals(Object.keys(data.schemas).length, 1);
    assertEquals(data.schemas["schema-1"].name, "TestSchema");
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("cacheBaseline() - writes YAML format correctly", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set up baseline and schema cache
  const components: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "test-component",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: {},
    },
  ];
  ctx.baseline(components);

  const schemaCache = ctx.schemaCache();
  schemaCache.set("schema-1", {
    schemaId: "schema-1",
    name: "TestSchema",
    defaultVariantId: "variant-1",
    variantIds: ["variant-1"],
  });

  // Cache to temp file
  const tempFile = await Deno.makeTempFile({ suffix: ".yaml" });
  try {
    await cacheBaseline(ctx, tempFile);

    // Verify file was written
    const content = await Deno.readTextFile(tempFile);
    const data = parseYaml(content) as BaselineCache;

    assertEquals(data.components.length, 1);
    assertEquals(data.components[0].id, "comp-1");
    assertEquals(Object.keys(data.schemas).length, 1);
    assertEquals(data.schemas["schema-1"].name, "TestSchema");
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("cacheBaseline() - handles undefined baseline gracefully", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // No baseline set
  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  try {
    await cacheBaseline(ctx, tempFile);

    // File should not be written
    const stat = await Deno.stat(tempFile);
    assertEquals(stat.size, 0); // Should be empty temp file
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("cacheBaseline() - handles empty baseline and schema cache", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set empty baseline
  ctx.baseline([]);

  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  try {
    await cacheBaseline(ctx, tempFile);

    const content = await Deno.readTextFile(tempFile);
    const data = JSON.parse(content) as BaselineCache;

    assertEquals(data.components.length, 0);
    assertEquals(Object.keys(data.schemas).length, 0);
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("cacheBaseline() - includes multiple schemas correctly", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  ctx.baseline([]);

  const schemaCache = ctx.schemaCache();
  schemaCache.set("schema-1", {
    schemaId: "schema-1",
    name: "Schema1",
    defaultVariantId: "v1",
    variantIds: ["v1"],
  });
  schemaCache.set("schema-2", {
    schemaId: "schema-2",
    name: "Schema2",
    defaultVariantId: "v2",
    variantIds: ["v2"],
  });

  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  try {
    await cacheBaseline(ctx, tempFile);

    const content = await Deno.readTextFile(tempFile);
    const data = JSON.parse(content) as BaselineCache;

    assertEquals(Object.keys(data.schemas).length, 2);
    assertEquals(data.schemas["schema-1"].name, "Schema1");
    assertEquals(data.schemas["schema-2"].name, "Schema2");
  } finally {
    await Deno.remove(tempFile);
  }
});
