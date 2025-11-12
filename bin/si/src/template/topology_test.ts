import { assertEquals, assertThrows } from "@std/assert";
import { topologicalSort } from "./topology.ts";
import { TemplateContext } from "./context.ts";
import type {
  CreateChange,
  DeleteChange,
  PendingChanges,
  UpdateChange,
} from "./converge_types.ts";
import type { TemplateComponent } from "./context.ts";
import { Context } from "../context.ts";

// Initialize Context for testing
Context.init({ verbose: 0, noColor: true });

function createMockContext(): TemplateContext {
  return new TemplateContext("test-template.ts", {
    key: "test-key",
  });
}

function createMockWorkingSetComponent(
  id: string,
  name: string,
): TemplateComponent {
  return {
    id,
    schemaId: "schema-123",
    name,
    resourceId: "resource-123",
    attributes: {},
  };
}

Deno.test("topologicalSort - no dependencies preserves original order", () => {
  const ctx = createMockContext();

  const create1: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp1", "Component 1"),
    attributes: {},
    dependencies: [],
  };

  const create2: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp2", "Component 2"),
    attributes: {},
    dependencies: [],
  };

  const pending: PendingChanges = {
    creates: [create1, create2],
    updates: [],
    deletes: [],
    workingSetById: new Map(),
    existingByWorkingSetId: new Map(),
    existingByDynamicName: new Map(),
  };

  const result = topologicalSort(ctx, pending);

  assertEquals(result.length, 2);
  assertEquals(result[0], create1);
  assertEquals(result[1], create2);
});

Deno.test("topologicalSort - linear dependency chain correct order", () => {
  const ctx = createMockContext();

  // comp3 depends on comp2, comp2 depends on comp1
  const create1: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp1", "Component 1"),
    attributes: {},
    dependencies: [],
  };

  const create2: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp2", "Component 2"),
    attributes: {},
    dependencies: ["comp1"],
  };

  const create3: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp3", "Component 3"),
    attributes: {},
    dependencies: ["comp2"],
  };

  const pending: PendingChanges = {
    creates: [create3, create2, create1], // Deliberately out of order
    updates: [],
    deletes: [],
    workingSetById: new Map(),
    existingByWorkingSetId: new Map(),
    existingByDynamicName: new Map(),
  };

  const result = topologicalSort(ctx, pending);

  assertEquals(result.length, 3);
  assertEquals(
    result[0].type === "create" || result[0].type === "update"
      ? result[0].workingSetComponent.id
      : undefined,
    "comp1",
  );
  assertEquals(
    result[1].type === "create" || result[1].type === "update"
      ? result[1].workingSetComponent.id
      : undefined,
    "comp2",
  );
  assertEquals(
    result[2].type === "create" || result[2].type === "update"
      ? result[2].workingSetComponent.id
      : undefined,
    "comp3",
  );
});

Deno.test("topologicalSort - diamond dependency correct order", () => {
  const ctx = createMockContext();

  // comp4 depends on comp2 and comp3, both depend on comp1
  //     comp1
  //    /     \
  //  comp2   comp3
  //    \     /
  //     comp4

  const create1: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp1", "Component 1"),
    attributes: {},
    dependencies: [],
  };

  const create2: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp2", "Component 2"),
    attributes: {},
    dependencies: ["comp1"],
  };

  const create3: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp3", "Component 3"),
    attributes: {},
    dependencies: ["comp1"],
  };

  const create4: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp4", "Component 4"),
    attributes: {},
    dependencies: ["comp2", "comp3"],
  };

  const pending: PendingChanges = {
    creates: [create4, create3, create2, create1], // Deliberately out of order
    updates: [],
    deletes: [],
    workingSetById: new Map(),
    existingByWorkingSetId: new Map(),
    existingByDynamicName: new Map(),
  };

  const result = topologicalSort(ctx, pending);

  assertEquals(result.length, 4);
  // comp1 must be first
  assertEquals(
    result[0].type === "create" || result[0].type === "update"
      ? result[0].workingSetComponent.id
      : undefined,
    "comp1",
  );
  // comp4 must be last
  assertEquals(
    result[3].type === "create" || result[3].type === "update"
      ? result[3].workingSetComponent.id
      : undefined,
    "comp4",
  );
  // comp2 and comp3 can be in either order, but both before comp4
  const comp2Index = result.findIndex((c) =>
    (c.type === "create" || c.type === "update") &&
    c.workingSetComponent.id === "comp2"
  );
  const comp3Index = result.findIndex((c) =>
    (c.type === "create" || c.type === "update") &&
    c.workingSetComponent.id === "comp3"
  );
  assertEquals(comp2Index > 0 && comp2Index < 3, true);
  assertEquals(comp3Index > 0 && comp3Index < 3, true);
});

Deno.test("topologicalSort - circular dependency throws error", () => {
  const ctx = createMockContext();

  // comp1 depends on comp2, comp2 depends on comp1
  const create1: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp1", "Component 1"),
    attributes: {},
    dependencies: ["comp2"],
  };

  const create2: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp2", "Component 2"),
    attributes: {},
    dependencies: ["comp1"],
  };

  const pending: PendingChanges = {
    creates: [create1, create2],
    updates: [],
    deletes: [],
    workingSetById: new Map(),
    existingByWorkingSetId: new Map(),
    existingByDynamicName: new Map(),
  };

  assertThrows(
    () => topologicalSort(ctx, pending),
    Error,
    "Circular dependency detected among components",
  );
});

Deno.test("topologicalSort - multiple independent chains all resolved", () => {
  const ctx = createMockContext();

  // Chain 1: comp1 -> comp2
  const create1: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp1", "Component 1"),
    attributes: {},
    dependencies: [],
  };

  const create2: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp2", "Component 2"),
    attributes: {},
    dependencies: ["comp1"],
  };

  // Chain 2: comp3 -> comp4
  const create3: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp3", "Component 3"),
    attributes: {},
    dependencies: [],
  };

  const create4: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp4", "Component 4"),
    attributes: {},
    dependencies: ["comp3"],
  };

  const pending: PendingChanges = {
    creates: [create2, create4, create1, create3], // Mixed order
    updates: [],
    deletes: [],
    workingSetById: new Map(),
    existingByWorkingSetId: new Map(),
    existingByDynamicName: new Map(),
  };

  const result = topologicalSort(ctx, pending);

  assertEquals(result.length, 4);

  // Check that dependencies are satisfied
  const positions = new Map<string, number>();
  result.forEach((change, idx) => {
    if (change.type === "create" || change.type === "update") {
      positions.set(change.workingSetComponent.id, idx);
    }
  });

  // comp1 before comp2
  assertEquals(
    positions.get("comp1")! < positions.get("comp2")!,
    true,
  );

  // comp3 before comp4
  assertEquals(
    positions.get("comp3")! < positions.get("comp4")!,
    true,
  );
});

Deno.test("topologicalSort - updates with dependencies sorted correctly", () => {
  const ctx = createMockContext();

  const create1: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp1", "Component 1"),
    attributes: {},
    dependencies: [],
  };

  const update1: UpdateChange = {
    type: "update",
    existingComponent: {
      id: "existing-2",
      schemaId: "schema-123",
      name: "Component 2",
      resourceId: "resource-123",
      attributes: {},
      templateWorkingSetId: "comp2",
    },
    workingSetComponent: createMockWorkingSetComponent("comp2", "Component 2"),
    attributeDiff: {
      set: new Map(),
      unset: [],
      subscriptions: new Map(),
    },
    dependencies: ["comp1"],
  };

  const pending: PendingChanges = {
    creates: [create1],
    updates: [update1],
    deletes: [],
    workingSetById: new Map(),
    existingByWorkingSetId: new Map(),
    existingByDynamicName: new Map(),
  };

  const result = topologicalSort(ctx, pending);

  assertEquals(result.length, 2);
  assertEquals(result[0].type, "create");
  assertEquals(result[1].type, "update");
});

Deno.test("topologicalSort - deletes always last", () => {
  const ctx = createMockContext();

  const create1: CreateChange = {
    type: "create",
    workingSetComponent: createMockWorkingSetComponent("comp1", "Component 1"),
    attributes: {},
    dependencies: [],
  };

  const delete1: DeleteChange = {
    type: "delete",
    existingComponent: {
      id: "existing-1",
      schemaId: "schema-123",
      name: "Old Component",
      resourceId: "resource-123",
      attributes: {},
      templateWorkingSetId: "old-comp",
    },
  };

  const pending: PendingChanges = {
    creates: [create1],
    updates: [],
    deletes: [delete1],
    workingSetById: new Map(),
    existingByWorkingSetId: new Map(),
    existingByDynamicName: new Map(),
  };

  const result = topologicalSort(ctx, pending);

  assertEquals(result.length, 2);
  assertEquals(result[0].type, "create");
  assertEquals(result[1].type, "delete");
});
