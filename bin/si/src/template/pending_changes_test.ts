import { assertEquals } from "@std/assert";
import { Context } from "../context.ts";
import { TemplateContext } from "./context.ts";
import type { ExistingSetComponent } from "./converge_types.ts";
import { buildPendingChanges } from "./pending_changes.ts";

// Initialize context once at module load
Context.init({ verbose: 0, noColor: true });

Deno.test("buildPendingChanges - empty working set results in all deletes", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });
  tctx.workingSet([]);

  const existingSet: ExistingSetComponent[] = [
    {
      id: "existing-1",
      schemaId: "schema-1",
      name: "Component 1",
      resourceId: "resource-1",
      attributes: { "/si/name": "Component 1" },
      templateWorkingSetId: "ws-1",
    },
    {
      id: "existing-2",
      schemaId: "schema-2",
      name: "Component 2",
      resourceId: "resource-2",
      attributes: { "/si/name": "Component 2" },
      templateWorkingSetId: "ws-2",
    },
  ];

  const pending = buildPendingChanges(tctx, existingSet);

  assertEquals(pending.creates.length, 0);
  assertEquals(pending.updates.length, 0);
  assertEquals(pending.deletes.length, 2);
  assertEquals(pending.deletes[0].existingComponent.id, "existing-1");
  assertEquals(pending.deletes[1].existingComponent.id, "existing-2");
});

Deno.test("buildPendingChanges - new component in working set creates change", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });
  tctx.workingSet([
    {
      id: "ws-1",
      schemaId: "schema-1",
      name: "New Component",
      resourceId: "resource-1",
      attributes: { "/si/name": "New Component" },
    },
  ]);

  const existingSet: ExistingSetComponent[] = [];

  const pending = buildPendingChanges(tctx, existingSet);

  assertEquals(pending.creates.length, 1);
  assertEquals(pending.updates.length, 0);
  assertEquals(pending.deletes.length, 0);
  assertEquals(pending.creates[0].type, "create");
  assertEquals(pending.creates[0].workingSetComponent.id, "ws-1");
  assertEquals(pending.creates[0].workingSetComponent.name, "New Component");
  assertEquals(pending.creates[0].dependencies.length, 0);
});

Deno.test("buildPendingChanges - existing component with no changes results in no update", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });
  tctx.workingSet([
    {
      id: "ws-1",
      schemaId: "schema-1",
      name: "Component 1",
      resourceId: "resource-1",
      attributes: { "/si/name": "Component 1", "/domain/value": "test" },
    },
  ]);

  const existingSet: ExistingSetComponent[] = [
    {
      id: "existing-1",
      schemaId: "schema-1",
      name: "Component 1",
      resourceId: "resource-1",
      attributes: { "/si/name": "Component 1", "/domain/value": "test" },
      templateWorkingSetId: "ws-1",
    },
  ];

  const pending = buildPendingChanges(tctx, existingSet);

  assertEquals(pending.creates.length, 0);
  assertEquals(pending.updates.length, 0);
  assertEquals(pending.deletes.length, 0);
});

Deno.test("buildPendingChanges - existing component with attribute changes creates update", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });
  tctx.workingSet([
    {
      id: "ws-1",
      schemaId: "schema-1",
      name: "Component 1",
      resourceId: "resource-1",
      attributes: { "/si/name": "Component 1", "/domain/value": "updated" },
    },
  ]);

  const existingSet: ExistingSetComponent[] = [
    {
      id: "existing-1",
      schemaId: "schema-1",
      name: "Component 1",
      resourceId: "resource-1",
      attributes: { "/si/name": "Component 1", "/domain/value": "original" },
      templateWorkingSetId: "ws-1",
    },
  ];

  const pending = buildPendingChanges(tctx, existingSet);

  assertEquals(pending.creates.length, 0);
  assertEquals(pending.updates.length, 1);
  assertEquals(pending.deletes.length, 0);
  assertEquals(pending.updates[0].type, "update");
  assertEquals(pending.updates[0].existingComponent.id, "existing-1");
  assertEquals(pending.updates[0].workingSetComponent.id, "ws-1");
  assertEquals(pending.updates[0].attributeDiff.set.size, 1);
  assertEquals(
    pending.updates[0].attributeDiff.set.get("/domain/value"),
    "updated",
  );
  assertEquals(pending.updates[0].nameChange, undefined);
});

Deno.test("buildPendingChanges - existing component with name change creates update", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });
  tctx.workingSet([
    {
      id: "ws-1",
      schemaId: "schema-1",
      name: "Component Renamed",
      resourceId: "resource-1",
      attributes: { "/si/name": "Component Renamed" },
    },
  ]);

  const existingSet: ExistingSetComponent[] = [
    {
      id: "existing-1",
      schemaId: "schema-1",
      name: "Component 1",
      resourceId: "resource-1",
      attributes: { "/si/name": "Component 1" },
      templateWorkingSetId: "ws-1",
    },
  ];

  const pending = buildPendingChanges(tctx, existingSet);

  assertEquals(pending.creates.length, 0);
  assertEquals(pending.updates.length, 1);
  assertEquals(pending.deletes.length, 0);
  assertEquals(pending.updates[0].nameChange?.from, "Component 1");
  assertEquals(pending.updates[0].nameChange?.to, "Component Renamed");
});

Deno.test("buildPendingChanges - component removed from working set creates delete", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });
  tctx.workingSet([]);

  const existingSet: ExistingSetComponent[] = [
    {
      id: "existing-1",
      schemaId: "schema-1",
      name: "Component 1",
      resourceId: "resource-1",
      attributes: { "/si/name": "Component 1" },
      templateWorkingSetId: "ws-1",
    },
  ];

  const pending = buildPendingChanges(tctx, existingSet);

  assertEquals(pending.creates.length, 0);
  assertEquals(pending.updates.length, 0);
  assertEquals(pending.deletes.length, 1);
  assertEquals(pending.deletes[0].type, "delete");
  assertEquals(pending.deletes[0].existingComponent.id, "existing-1");
});

Deno.test("buildPendingChanges - multiple components with mixed operations", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });
  tctx.workingSet([
    {
      id: "ws-1",
      schemaId: "schema-1",
      name: "Component 1",
      resourceId: "resource-1",
      attributes: { "/si/name": "Component 1", "/domain/value": "updated" },
    },
    {
      id: "ws-2",
      schemaId: "schema-2",
      name: "New Component",
      resourceId: "resource-2",
      attributes: { "/si/name": "New Component" },
    },
    {
      id: "ws-3",
      schemaId: "schema-3",
      name: "Component 3",
      resourceId: "resource-3",
      attributes: { "/si/name": "Component 3" },
    },
  ]);

  const existingSet: ExistingSetComponent[] = [
    {
      id: "existing-1",
      schemaId: "schema-1",
      name: "Component 1",
      resourceId: "resource-1",
      attributes: { "/si/name": "Component 1", "/domain/value": "original" },
      templateWorkingSetId: "ws-1",
    },
    {
      id: "existing-3",
      schemaId: "schema-3",
      name: "Component 3",
      resourceId: "resource-3",
      attributes: { "/si/name": "Component 3" },
      templateWorkingSetId: "ws-3",
    },
    {
      id: "existing-4",
      schemaId: "schema-4",
      name: "Component 4",
      resourceId: "resource-4",
      attributes: { "/si/name": "Component 4" },
      templateWorkingSetId: "ws-4",
    },
  ];

  const pending = buildPendingChanges(tctx, existingSet);

  assertEquals(pending.creates.length, 1, "Should have 1 create");
  assertEquals(pending.updates.length, 1, "Should have 1 update");
  assertEquals(pending.deletes.length, 1, "Should have 1 delete");

  // Check create
  assertEquals(pending.creates[0].workingSetComponent.id, "ws-2");
  assertEquals(pending.creates[0].workingSetComponent.name, "New Component");

  // Check update
  assertEquals(pending.updates[0].workingSetComponent.id, "ws-1");
  assertEquals(
    pending.updates[0].attributeDiff.set.get("/domain/value"),
    "updated",
  );

  // Check delete
  assertEquals(pending.deletes[0].existingComponent.id, "existing-4");

  // Check lookup maps
  assertEquals(pending.workingSetById.size, 3);
  assertEquals(pending.existingByWorkingSetId.size, 3);
});

Deno.test("buildPendingChanges - dynamic component matched by name on second run (idempotent)", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });

  // First run: dynamic component with new ULID (simulating copyComponent)
  tctx.workingSet([
    {
      id: "ws-new-ulid-123",
      schemaId: "schema-1",
      name: "server-1",
      resourceId: "resource-1",
      attributes: {
        "/si/name": "server-1",
        "/si/tags/templateDynamicName": "server-1",
        "/domain/port": 8080,
      },
    },
  ]);

  // Existing set has the same component but with different working set ID
  const existingSet: ExistingSetComponent[] = [
    {
      id: "existing-si-id-456",
      schemaId: "schema-1",
      name: "server-1",
      resourceId: "resource-1",
      attributes: {
        "/si/name": "server-1",
        "/si/tags/templateDynamicName": "server-1",
        "/domain/port": 8080,
      },
      templateWorkingSetId: "ws-old-ulid-789",
    },
  ];

  const pending = buildPendingChanges(tctx, existingSet);

  // Should be detected as UPDATE (no changes), not DELETE + CREATE
  assertEquals(pending.creates.length, 0, "Should have 0 creates");
  assertEquals(
    pending.updates.length,
    0,
    "Should have 0 updates (no attribute changes)",
  );
  assertEquals(pending.deletes.length, 0, "Should have 0 deletes");

  // Verify that the working set ID remains unchanged to preserve subscription references
  assertEquals(
    tctx.workingSet()![0].id,
    "ws-new-ulid-123",
    "Working set ID should remain unchanged to preserve subscriptions",
  );

  // Verify that the mapping was added for subscription rewriting
  assertEquals(
    pending.existingByWorkingSetId.has("ws-new-ulid-123"),
    true,
    "Mapping should be added to existingByWorkingSetId",
  );
});

Deno.test("buildPendingChanges - dynamic component matched by name with attribute changes", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });

  tctx.workingSet([
    {
      id: "ws-new-ulid-123",
      schemaId: "schema-1",
      name: "server-1",
      resourceId: "resource-1",
      attributes: {
        "/si/name": "server-1",
        "/si/tags/templateDynamicName": "server-1",
        "/domain/port": 9090, // Changed from 8080
      },
    },
  ]);

  const existingSet: ExistingSetComponent[] = [
    {
      id: "existing-si-id-456",
      schemaId: "schema-1",
      name: "server-1",
      resourceId: "resource-1",
      attributes: {
        "/si/name": "server-1",
        "/si/tags/templateDynamicName": "server-1",
        "/domain/port": 8080,
      },
      templateWorkingSetId: "ws-old-ulid-789",
    },
  ];

  const pending = buildPendingChanges(tctx, existingSet);

  // Should be UPDATE with attribute change
  assertEquals(pending.creates.length, 0, "Should have 0 creates");
  assertEquals(pending.updates.length, 1, "Should have 1 update");
  assertEquals(pending.deletes.length, 0, "Should have 0 deletes");

  assertEquals(pending.updates[0].type, "update");
  assertEquals(pending.updates[0].existingComponent.id, "existing-si-id-456");
  assertEquals(pending.updates[0].attributeDiff.set.get("/domain/port"), 9090);
});

Deno.test("buildPendingChanges - dynamic component name changed results in delete + create", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });

  // Working set has component with NEW name
  tctx.workingSet([
    {
      id: "ws-new-ulid-123",
      schemaId: "schema-1",
      name: "server-2",
      resourceId: "resource-1",
      attributes: {
        "/si/name": "server-2",
        "/si/tags/templateDynamicName": "server-2", // Changed name
        "/domain/port": 8080,
      },
    },
  ]);

  // Existing set has component with OLD name
  const existingSet: ExistingSetComponent[] = [
    {
      id: "existing-si-id-456",
      schemaId: "schema-1",
      name: "server-1",
      resourceId: "resource-1",
      attributes: {
        "/si/name": "server-1",
        "/si/tags/templateDynamicName": "server-1", // Old name
        "/domain/port": 8080,
      },
      templateWorkingSetId: "ws-old-ulid-789",
    },
  ];

  const pending = buildPendingChanges(tctx, existingSet);

  // Should be DELETE + CREATE (name mismatch means no match)
  assertEquals(pending.creates.length, 1, "Should have 1 create");
  assertEquals(pending.updates.length, 0, "Should have 0 updates");
  assertEquals(pending.deletes.length, 1, "Should have 1 delete");

  assertEquals(pending.creates[0].workingSetComponent.name, "server-2");
  assertEquals(pending.deletes[0].existingComponent.name, "server-1");
});

Deno.test("buildPendingChanges - regular components still match by ID (no regression)", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });

  // Regular component without dynamic name tag
  tctx.workingSet([
    {
      id: "ws-1",
      schemaId: "schema-1",
      name: "Regular Component",
      resourceId: "resource-1",
      attributes: {
        "/si/name": "Regular Component",
        "/domain/value": "test",
      },
    },
  ]);

  const existingSet: ExistingSetComponent[] = [
    {
      id: "existing-1",
      schemaId: "schema-1",
      name: "Regular Component",
      resourceId: "resource-1",
      attributes: {
        "/si/name": "Regular Component",
        "/domain/value": "test",
      },
      templateWorkingSetId: "ws-1",
    },
  ];

  const pending = buildPendingChanges(tctx, existingSet);

  // Should match normally by ID
  assertEquals(pending.creates.length, 0);
  assertEquals(pending.updates.length, 0);
  assertEquals(pending.deletes.length, 0);
});

Deno.test("buildPendingChanges - mixed regular and dynamic components", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });

  tctx.workingSet([
    // Regular component
    {
      id: "ws-regular",
      schemaId: "schema-1",
      name: "Regular",
      resourceId: "resource-1",
      attributes: { "/si/name": "Regular" },
    },
    // Dynamic component (matched by name)
    {
      id: "ws-new-dynamic-id",
      schemaId: "schema-2",
      name: "dynamic-server",
      resourceId: "resource-2",
      attributes: {
        "/si/name": "dynamic-server",
        "/si/tags/templateDynamicName": "dynamic-server",
      },
    },
  ]);

  const existingSet: ExistingSetComponent[] = [
    // Regular component matches by ID
    {
      id: "existing-regular",
      schemaId: "schema-1",
      name: "Regular",
      resourceId: "resource-1",
      attributes: { "/si/name": "Regular" },
      templateWorkingSetId: "ws-regular",
    },
    // Dynamic component matches by name (different ID)
    {
      id: "existing-dynamic",
      schemaId: "schema-2",
      name: "dynamic-server",
      resourceId: "resource-2",
      attributes: {
        "/si/name": "dynamic-server",
        "/si/tags/templateDynamicName": "dynamic-server",
      },
      templateWorkingSetId: "ws-old-dynamic-id",
    },
  ];

  const pending = buildPendingChanges(tctx, existingSet);

  // Both should match (no creates or deletes)
  assertEquals(pending.creates.length, 0);
  assertEquals(pending.updates.length, 0);
  assertEquals(pending.deletes.length, 0);

  // Check that IDs remain unchanged to preserve subscription references
  const workingSet = tctx.workingSet()!;
  assertEquals(
    workingSet[0].id,
    "ws-regular",
    "Regular component ID unchanged",
  );
  assertEquals(
    workingSet[1].id,
    "ws-new-dynamic-id",
    "Dynamic component ID unchanged to preserve subscriptions",
  );

  // Verify that mappings were added for subscription rewriting
  assertEquals(
    pending.existingByWorkingSetId.has("ws-regular"),
    true,
    "Regular component mapping exists",
  );
  assertEquals(
    pending.existingByWorkingSetId.has("ws-new-dynamic-id"),
    true,
    "Dynamic component mapping exists",
  );
});

Deno.test("buildPendingChanges - existingByDynamicName map is populated", () => {
  const tctx = new TemplateContext("/tmp/test.ts", { key: "test" });
  tctx.workingSet([]);

  const existingSet: ExistingSetComponent[] = [
    {
      id: "existing-1",
      schemaId: "schema-1",
      name: "server-1",
      resourceId: "resource-1",
      attributes: {
        "/si/name": "server-1",
        "/si/tags/templateDynamicName": "server-1",
      },
      templateWorkingSetId: "ws-1",
    },
    {
      id: "existing-2",
      schemaId: "schema-2",
      name: "server-2",
      resourceId: "resource-2",
      attributes: {
        "/si/name": "server-2",
        "/si/tags/templateDynamicName": "server-2",
      },
      templateWorkingSetId: "ws-2",
    },
    {
      id: "existing-3",
      schemaId: "schema-3",
      name: "regular",
      resourceId: "resource-3",
      attributes: {
        "/si/name": "regular",
        // No templateDynamicName tag
      },
      templateWorkingSetId: "ws-3",
    },
  ];

  const pending = buildPendingChanges(tctx, existingSet);

  // Check that the dynamic name lookup map is populated correctly
  assertEquals(
    pending.existingByDynamicName.size,
    2,
    "Should have 2 dynamic components",
  );
  assertEquals(
    pending.existingByDynamicName.get("server-1")?.id,
    "existing-1",
    "server-1 should be in map",
  );
  assertEquals(
    pending.existingByDynamicName.get("server-2")?.id,
    "existing-2",
    "server-2 should be in map",
  );
  assertEquals(
    pending.existingByDynamicName.has("regular"),
    false,
    "regular component should not be in map",
  );
});
