import { assertEquals, assertExists } from "@std/assert";
import { Context } from "../context.ts";
import { TemplateContext } from "./context.ts";
import { loadBaselineFromFile } from "./baseline.ts";
import type { BaselineCache } from "./cache.ts";

// Initialize Context once - this runs at module load time
await Context.init({ verbose: 0, noColor: true });

Deno.test("loadBaselineFromFile() - loads cache format with components and schemas", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Create temp cache file
  const cacheData: BaselineCache = {
    components: [
      {
        id: "comp-1",
        name: "test-component",
        schemaId: "schema-1",
        resourceId: "res-1",
        attributes: {},
      },
    ],
    schemas: {
      "schema-1": {
        schemaId: "schema-1",
        name: "TestSchema",
        defaultVariantId: "variant-1",
        variantIds: ["variant-1"],
      },
    },
  };

  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  try {
    await Deno.writeTextFile(tempFile, JSON.stringify(cacheData));

    await loadBaselineFromFile(ctx, tempFile);

    // Verify baseline was set
    const baseline = ctx.baseline();
    assertExists(baseline);
    assertEquals(baseline.length, 1);
    assertEquals(baseline[0].id, "comp-1");

    // Verify schema cache was populated
    const schemaCache = ctx.schemaCache();
    assertEquals(schemaCache.size, 1);
    assertEquals(schemaCache.get("schema-1")?.name, "TestSchema");
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("loadBaselineFromFile() - populates schema cache correctly from YAML", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const cacheData: BaselineCache = {
    components: [],
    schemas: {
      "schema-1": {
        schemaId: "schema-1",
        name: "Schema1",
        defaultVariantId: "v1",
        variantIds: ["v1"],
      },
      "schema-2": {
        schemaId: "schema-2",
        name: "Schema2",
        defaultVariantId: "v2",
        variantIds: ["v2"],
      },
    },
  };

  const tempFile = await Deno.makeTempFile({ suffix: ".yaml" });
  try {
    await Deno.writeTextFile(tempFile, JSON.stringify(cacheData));

    await loadBaselineFromFile(ctx, tempFile);

    const schemaCache = ctx.schemaCache();
    assertEquals(schemaCache.size, 2);
    assertEquals(schemaCache.get("schema-1")?.name, "Schema1");
    assertEquals(schemaCache.get("schema-2")?.name, "Schema2");
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("loadBaselineFromFile() - handles multiple components correctly", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const cacheData: BaselineCache = {
    components: [
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
        schemaId: "schema-1",
        resourceId: "res-2",
        attributes: {},
      },
      {
        id: "comp-3",
        name: "component-3",
        schemaId: "schema-2",
        resourceId: "res-3",
        attributes: {},
      },
    ],
    schemas: {},
  };

  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  try {
    await Deno.writeTextFile(tempFile, JSON.stringify(cacheData));

    await loadBaselineFromFile(ctx, tempFile);

    const baseline = ctx.baseline();
    assertExists(baseline);
    assertEquals(baseline.length, 3);
    assertEquals(baseline[0].id, "comp-1");
    assertEquals(baseline[1].id, "comp-2");
    assertEquals(baseline[2].id, "comp-3");
  } finally {
    await Deno.remove(tempFile);
  }
});
