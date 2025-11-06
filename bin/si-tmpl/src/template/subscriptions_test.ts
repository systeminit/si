import { assertEquals } from "@std/assert";
import { Context } from "../context.ts";
import { TemplateContext } from "./context.ts";
import { rewriteSubscriptions } from "./subscriptions.ts";
import type { PendingChanges, UpdateChange } from "./converge_types.ts";
import type { TemplateComponent } from "./context.ts";

// Initialize context once at module load
Context.init({ verbose: 0, noColor: true });

// Helper to create a test context
function createTestContext(): TemplateContext {
  return new TemplateContext("/tmp/test.ts", { key: "test" });
}

// Helper to create a template component
function createTemplateComponent(
  id: string,
  name: string,
  attributes: { [key: string]: unknown } = {},
): TemplateComponent {
  return {
    id,
    schemaId: "schema-123",
    name,
    resourceId: "resource-123",
    attributes,
  };
}

Deno.test("rewriteSubscriptions - subscription to existing component is rewritten to SI ID", () => {
  const ctx = createTestContext();

  // Create a component that exists in SI
  const existingComp = {
    id: "si-id-123",
    schemaId: "schema-123",
    name: "existing-comp",
    resourceId: "resource-123",
    attributes: {},
    templateWorkingSetId: "ws-existing",
  };

  // Create a new component that subscribes to the existing one
  const newComp = createTemplateComponent("ws-new", "new-comp", {
    "/domain/config": {
      $source: {
        component: "ws-existing",
        path: "/domain/output",
      },
    },
  });

  const pending: PendingChanges = {
    creates: [{
      type: "create",
      workingSetComponent: newComp,
      attributes: newComp.attributes,
      dependencies: [],
    }],
    updates: [],
    deletes: [],
    workingSetById: new Map([
      ["ws-existing", createTemplateComponent("ws-existing", "existing-comp")],
      ["ws-new", newComp],
    ]),
    existingByWorkingSetId: new Map([
      ["ws-existing", existingComp],
    ]),
    existingByDynamicName: new Map(),
  };

  const result = rewriteSubscriptions(ctx, pending);

  // The subscription should be rewritten to use the SI ID
  const create = result.creates[0];
  assertEquals(
    create.attributes["/domain/config"].$source.component,
    "si-id-123",
  );
  assertEquals(create.dependencies, []);
});

Deno.test("rewriteSubscriptions - subscription to component being created is added as dependency", () => {
  const ctx = createTestContext();

  // Create two new components where one depends on the other
  const compA = createTemplateComponent("ws-a", "comp-a", {
    "/domain/value": "hello",
  });

  const compB = createTemplateComponent("ws-b", "comp-b", {
    "/domain/config": {
      $source: {
        component: "ws-a",
        path: "/domain/value",
      },
    },
  });

  const pending: PendingChanges = {
    creates: [
      {
        type: "create",
        workingSetComponent: compA,
        attributes: compA.attributes,
        dependencies: [],
      },
      {
        type: "create",
        workingSetComponent: compB,
        attributes: compB.attributes,
        dependencies: [],
      },
    ],
    updates: [],
    deletes: [],
    workingSetById: new Map([
      ["ws-a", compA],
      ["ws-b", compB],
    ]),
    existingByWorkingSetId: new Map(),
    existingByDynamicName: new Map(),
  };

  const result = rewriteSubscriptions(ctx, pending);

  // compB should have compA as a dependency
  const createB = result.creates[1];
  assertEquals(createB.dependencies, ["ws-a"]);

  // The subscription should still reference the workingSet ID since it hasn't been created yet
  assertEquals(
    createB.attributes["/domain/config"].$source.component,
    "ws-a",
  );
});

Deno.test("rewriteSubscriptions - subscription by name is left unchanged", () => {
  const ctx = createTestContext();

  const comp = createTemplateComponent("ws-comp", "comp", {
    "/domain/config": {
      $source: {
        component: "some-component-name",
        path: "/domain/output",
      },
    },
  });

  const pending: PendingChanges = {
    creates: [{
      type: "create",
      workingSetComponent: comp,
      attributes: comp.attributes,
      dependencies: [],
    }],
    updates: [],
    deletes: [],
    workingSetById: new Map([["ws-comp", comp]]),
    existingByWorkingSetId: new Map(),
    existingByDynamicName: new Map(),
  };

  const result = rewriteSubscriptions(ctx, pending);

  // The subscription should remain unchanged
  const create = result.creates[0];
  assertEquals(
    create.attributes["/domain/config"].$source.component,
    "some-component-name",
  );
  assertEquals(create.dependencies, []);
});

Deno.test("rewriteSubscriptions - subscription by SI ID is left unchanged", () => {
  const ctx = createTestContext();

  const comp = createTemplateComponent("ws-comp", "comp", {
    "/domain/config": {
      $source: {
        component: "si-id-external-123",
        path: "/domain/output",
      },
    },
  });

  const pending: PendingChanges = {
    creates: [{
      type: "create",
      workingSetComponent: comp,
      attributes: comp.attributes,
      dependencies: [],
    }],
    updates: [],
    deletes: [],
    workingSetById: new Map([["ws-comp", comp]]),
    existingByWorkingSetId: new Map(),
    existingByDynamicName: new Map(),
  };

  const result = rewriteSubscriptions(ctx, pending);

  // The subscription should remain unchanged
  const create = result.creates[0];
  assertEquals(
    create.attributes["/domain/config"].$source.component,
    "si-id-external-123",
  );
  assertEquals(create.dependencies, []);
});

Deno.test("rewriteSubscriptions - multiple subscriptions in one component", () => {
  const ctx = createTestContext();

  const existingComp = {
    id: "si-id-existing",
    schemaId: "schema-123",
    name: "existing",
    resourceId: "resource-123",
    attributes: {},
    templateWorkingSetId: "ws-existing",
  };

  const newComp = createTemplateComponent("ws-new", "new", {});

  const comp = createTemplateComponent("ws-comp", "comp", {
    "/domain/config1": {
      $source: {
        component: "ws-existing",
        path: "/domain/output1",
      },
    },
    "/domain/config2": {
      $source: {
        component: "ws-new",
        path: "/domain/output2",
      },
    },
    "/domain/config3": {
      $source: {
        component: "external-name",
        path: "/domain/output3",
      },
    },
  });

  const pending: PendingChanges = {
    creates: [{
      type: "create",
      workingSetComponent: comp,
      attributes: comp.attributes,
      dependencies: [],
    }],
    updates: [],
    deletes: [],
    workingSetById: new Map([
      ["ws-existing", createTemplateComponent("ws-existing", "existing")],
      ["ws-new", newComp],
      ["ws-comp", comp],
    ]),
    existingByWorkingSetId: new Map([
      ["ws-existing", existingComp],
    ]),
    existingByDynamicName: new Map(),
  };

  const result = rewriteSubscriptions(ctx, pending);

  const create = result.creates[0];

  // Subscription to existing component should be rewritten
  assertEquals(
    create.attributes["/domain/config1"].$source.component,
    "si-id-existing",
  );

  // Subscription to new component should remain as workingSet ID and add dependency
  assertEquals(
    create.attributes["/domain/config2"].$source.component,
    "ws-new",
  );
  assertEquals(create.dependencies, ["ws-new"]);

  // Subscription by name should remain unchanged
  assertEquals(
    create.attributes["/domain/config3"].$source.component,
    "external-name",
  );
});

Deno.test("rewriteSubscriptions - no subscriptions results in no changes", () => {
  const ctx = createTestContext();

  const comp = createTemplateComponent("ws-comp", "comp", {
    "/domain/value": "hello",
    "/domain/number": 42,
  });

  const pending: PendingChanges = {
    creates: [{
      type: "create",
      workingSetComponent: comp,
      attributes: comp.attributes,
      dependencies: [],
    }],
    updates: [],
    deletes: [],
    workingSetById: new Map([["ws-comp", comp]]),
    existingByWorkingSetId: new Map(),
    existingByDynamicName: new Map(),
  };

  const result = rewriteSubscriptions(ctx, pending);

  const create = result.creates[0];
  assertEquals(create.attributes, comp.attributes);
  assertEquals(create.dependencies, []);
});

Deno.test("rewriteSubscriptions - subscription in update diff", () => {
  const ctx = createTestContext();

  const existingComp = {
    id: "si-id-123",
    schemaId: "schema-123",
    name: "existing",
    resourceId: "resource-123",
    attributes: {},
    templateWorkingSetId: "ws-existing",
  };

  const targetComp = {
    id: "si-id-target",
    schemaId: "schema-123",
    name: "target",
    resourceId: "resource-123",
    attributes: {},
    templateWorkingSetId: "ws-target",
  };

  const updateChange: UpdateChange = {
    type: "update",
    existingComponent: existingComp,
    workingSetComponent: createTemplateComponent("ws-existing", "existing"),
    attributeDiff: {
      set: new Map(),
      unset: [],
      subscriptions: new Map([
        ["/domain/config", {
          component: "ws-target",
          path: "/domain/output",
        }],
      ]),
    },
    dependencies: [],
  };

  const pending: PendingChanges = {
    creates: [],
    updates: [updateChange],
    deletes: [],
    workingSetById: new Map([
      ["ws-existing", createTemplateComponent("ws-existing", "existing")],
      ["ws-target", createTemplateComponent("ws-target", "target")],
    ]),
    existingByWorkingSetId: new Map([
      ["ws-existing", existingComp],
      ["ws-target", targetComp],
    ]),
    existingByDynamicName: new Map(),
  };

  const result = rewriteSubscriptions(ctx, pending);

  const update = result.updates[0];
  const subscription = update.attributeDiff.subscriptions.get("/domain/config");

  // Subscription should be rewritten to SI ID
  assertEquals(subscription?.component, "si-id-target");
  assertEquals(subscription?.path, "/domain/output");
  assertEquals(update.dependencies, []);
});

Deno.test("rewriteSubscriptions - subscription with transformation function", () => {
  const ctx = createTestContext();

  const existingComp = {
    id: "si-id-123",
    schemaId: "schema-123",
    name: "existing",
    resourceId: "resource-123",
    attributes: {},
    templateWorkingSetId: "ws-existing",
  };

  const comp = createTemplateComponent("ws-comp", "comp", {
    "/domain/config": {
      $source: {
        component: "ws-existing",
        path: "/domain/output",
        func: "toUpperCase",
      },
    },
  });

  const pending: PendingChanges = {
    creates: [{
      type: "create",
      workingSetComponent: comp,
      attributes: comp.attributes,
      dependencies: [],
    }],
    updates: [],
    deletes: [],
    workingSetById: new Map([
      ["ws-existing", createTemplateComponent("ws-existing", "existing")],
      ["ws-comp", comp],
    ]),
    existingByWorkingSetId: new Map([
      ["ws-existing", existingComp],
    ]),
    existingByDynamicName: new Map(),
  };

  const result = rewriteSubscriptions(ctx, pending);

  const create = result.creates[0];
  assertEquals(
    create.attributes["/domain/config"].$source.component,
    "si-id-123",
  );
  assertEquals(
    create.attributes["/domain/config"].$source.func,
    "toUpperCase",
  );
});
