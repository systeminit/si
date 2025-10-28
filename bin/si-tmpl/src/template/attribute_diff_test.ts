import { assertEquals } from "@std/assert";
import {
  attributeDiffToUpdatePayload,
  collapseArrayElementUnsets,
  computeAttributeDiff,
  isEmptyDiff,
} from "./attribute_diff.ts";

Deno.test("computeAttributeDiff - no changes when attributes identical", () => {
  const attrs = {
    "/domain/name": "test",
    "/domain/value": 42,
  };

  const diff = computeAttributeDiff(attrs, attrs);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.unset.length, 0);
  assertEquals(diff.subscriptions.size, 0);
  assertEquals(isEmptyDiff(diff), true);
});

Deno.test("computeAttributeDiff - detects simple value changes", () => {
  const workingSet = {
    "/domain/name": "new-name",
    "/domain/value": 100,
  };

  const existing = {
    "/domain/name": "old-name",
    "/domain/value": 42,
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 2);
  assertEquals(diff.set.get("/domain/name"), "new-name");
  assertEquals(diff.set.get("/domain/value"), 100);
  assertEquals(diff.unset.length, 0);
  assertEquals(diff.subscriptions.size, 0);
});

Deno.test("computeAttributeDiff - detects new attributes (set)", () => {
  const workingSet = {
    "/domain/name": "test",
    "/domain/new": "value",
  };

  const existing = {
    "/domain/name": "test",
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 1);
  assertEquals(diff.set.get("/domain/new"), "value");
  assertEquals(diff.unset.length, 0);
});

Deno.test("computeAttributeDiff - detects removed attributes (unset)", () => {
  const workingSet = {
    "/domain/name": "test",
  };

  const existing = {
    "/domain/name": "test",
    "/domain/old": "value",
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.unset.length, 1);
  assertEquals(diff.unset[0], "/domain/old");
});

Deno.test("computeAttributeDiff - detects nested object changes", () => {
  const workingSet = {
    "/domain/config": {
      host: "localhost",
      port: 8080,
    },
  };

  const existing = {
    "/domain/config": {
      host: "localhost",
      port: 3000,
    },
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 1);
  assertEquals(diff.set.get("/domain/config"), {
    host: "localhost",
    port: 8080,
  });
});

Deno.test("computeAttributeDiff - detects array changes", () => {
  const workingSet = {
    "/domain/items": [1, 2, 3],
  };

  const existing = {
    "/domain/items": [1, 2],
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 1);
  assertEquals(diff.set.get("/domain/items"), [1, 2, 3]);
});

Deno.test("computeAttributeDiff - identifies subscription objects", () => {
  const workingSet = {
    "/domain/name": "test",
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/value",
      },
    },
  };

  const existing = {
    "/domain/name": "test",
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.subscriptions.size, 1);
  assertEquals(diff.subscriptions.get("/domain/ref"), {
    component: "comp-123",
    path: "/domain/value",
    func: undefined,
  });
});

Deno.test("computeAttributeDiff - preserves template tags (doesn't unset them)", () => {
  const workingSet = {
    "/domain/name": "test",
  };

  const existing = {
    "/domain/name": "test",
    "/si/tags/templateFrom": "my-template",
    "/si/tags/templateWorkingSetId": "ws-123",
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.unset.length, 0); // Template tags should NOT be unset
});

Deno.test("attributeDiffToUpdatePayload - formats set operations correctly", () => {
  const diff = {
    set: new Map<string, unknown>([
      ["/domain/name", "test"],
      ["/domain/value", 42],
    ]),
    unset: [],
    subscriptions: new Map(),
  };

  const payload = attributeDiffToUpdatePayload(diff);

  assertEquals(payload, {
    "/domain/name": "test",
    "/domain/value": 42,
  });
});

Deno.test("attributeDiffToUpdatePayload - formats unset operations correctly", () => {
  const diff = {
    set: new Map(),
    unset: ["/domain/old", "/domain/deprecated"],
    subscriptions: new Map(),
  };

  const payload = attributeDiffToUpdatePayload(diff);

  assertEquals(payload, {
    "/domain/old": { "$source": null },
    "/domain/deprecated": { "$source": null },
  });
});

Deno.test("attributeDiffToUpdatePayload - formats subscriptions correctly", () => {
  const diff = {
    set: new Map(),
    unset: [],
    subscriptions: new Map([
      ["/domain/ref", {
        component: "comp-123",
        path: "/domain/value",
        func: undefined,
      }],
    ]),
  };

  const payload = attributeDiffToUpdatePayload(diff);

  assertEquals(payload, {
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/value",
      },
    },
  });
});

Deno.test("attributeDiffToUpdatePayload - includes func when present", () => {
  const diff = {
    set: new Map(),
    unset: [],
    subscriptions: new Map([
      ["/domain/ref", {
        component: "comp-123",
        path: "/domain/value",
        func: "si:transform",
      }],
    ]),
  };

  const payload = attributeDiffToUpdatePayload(diff);

  assertEquals(payload, {
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/value",
        func: "si:transform",
      },
    },
  });
});

Deno.test("attributeDiffToUpdatePayload - combines all operation types", () => {
  const diff = {
    set: new Map<string, unknown>([["/domain/name", "test"]]),
    unset: ["/domain/old"],
    subscriptions: new Map([
      ["/domain/ref", {
        component: "comp-123",
        path: "/domain/value",
        func: undefined,
      }],
    ]),
  };

  const payload = attributeDiffToUpdatePayload(diff);

  assertEquals(payload, {
    "/domain/name": "test",
    "/domain/old": { "$source": null },
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/value",
      },
    },
  });
});

Deno.test("isEmptyDiff - correctly identifies empty diffs", () => {
  const emptyDiff = {
    set: new Map(),
    unset: [],
    subscriptions: new Map(),
  };

  assertEquals(isEmptyDiff(emptyDiff), true);
});

Deno.test("isEmptyDiff - correctly identifies non-empty diffs (set)", () => {
  const diff = {
    set: new Map([["/domain/name", "test"]]),
    unset: [],
    subscriptions: new Map(),
  };

  assertEquals(isEmptyDiff(diff), false);
});

Deno.test("isEmptyDiff - correctly identifies non-empty diffs (unset)", () => {
  const diff = {
    set: new Map(),
    unset: ["/domain/old"],
    subscriptions: new Map(),
  };

  assertEquals(isEmptyDiff(diff), false);
});

Deno.test("isEmptyDiff - correctly identifies non-empty diffs (subscriptions)", () => {
  const diff = {
    set: new Map(),
    unset: [],
    subscriptions: new Map([
      ["/domain/ref", {
        component: "comp-123",
        path: "/domain/value",
        func: undefined,
      }],
    ]),
  };

  assertEquals(isEmptyDiff(diff), false);
});

Deno.test("computeAttributeDiff - subscription unchanged (identical)", () => {
  const subscription = {
    "$source": {
      component: "comp-123",
      path: "/domain/value",
    },
  };

  const workingSet = {
    "/domain/name": "test",
    "/domain/ref": subscription,
  };

  const existing = {
    "/domain/name": "test",
    "/domain/ref": subscription,
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.unset.length, 0);
  assertEquals(diff.subscriptions.size, 0);
  assertEquals(isEmptyDiff(diff), true);
});

Deno.test("computeAttributeDiff - subscription component changed", () => {
  const workingSet = {
    "/domain/ref": {
      "$source": {
        component: "comp-new",
        path: "/domain/value",
      },
    },
  };

  const existing = {
    "/domain/ref": {
      "$source": {
        component: "comp-old",
        path: "/domain/value",
      },
    },
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.subscriptions.size, 1);
  assertEquals(diff.subscriptions.get("/domain/ref"), {
    component: "comp-new",
    path: "/domain/value",
    func: undefined,
  });
});

Deno.test("computeAttributeDiff - subscription path changed", () => {
  const workingSet = {
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/new-path",
      },
    },
  };

  const existing = {
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/old-path",
      },
    },
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.subscriptions.size, 1);
  assertEquals(diff.subscriptions.get("/domain/ref"), {
    component: "comp-123",
    path: "/domain/new-path",
    func: undefined,
  });
});

Deno.test("computeAttributeDiff - subscription func changed", () => {
  const workingSet = {
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/value",
        func: "newFunc",
      },
    },
  };

  const existing = {
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/value",
        func: "oldFunc",
      },
    },
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.subscriptions.size, 1);
  assertEquals(diff.subscriptions.get("/domain/ref"), {
    component: "comp-123",
    path: "/domain/value",
    func: "newFunc",
  });
});

Deno.test("computeAttributeDiff - subscription func added", () => {
  const workingSet = {
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/value",
        func: "newFunc",
      },
    },
  };

  const existing = {
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/value",
      },
    },
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.subscriptions.size, 1);
  assertEquals(diff.subscriptions.get("/domain/ref"), {
    component: "comp-123",
    path: "/domain/value",
    func: "newFunc",
  });
});

Deno.test("computeAttributeDiff - subscription func removed", () => {
  const workingSet = {
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/value",
      },
    },
  };

  const existing = {
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/value",
        func: "oldFunc",
      },
    },
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.subscriptions.size, 1);
  assertEquals(diff.subscriptions.get("/domain/ref"), {
    component: "comp-123",
    path: "/domain/value",
    func: undefined,
  });
});

Deno.test("computeAttributeDiff - subscription replaces regular value", () => {
  const workingSet = {
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/value",
      },
    },
  };

  const existing = {
    "/domain/ref": "some-regular-value",
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.subscriptions.size, 1);
  assertEquals(diff.subscriptions.get("/domain/ref"), {
    component: "comp-123",
    path: "/domain/value",
    func: undefined,
  });
});

Deno.test("computeAttributeDiff - regular value replaces subscription", () => {
  const workingSet = {
    "/domain/ref": "new-regular-value",
  };

  const existing = {
    "/domain/ref": {
      "$source": {
        component: "comp-123",
        path: "/domain/value",
      },
    },
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 1);
  assertEquals(diff.set.get("/domain/ref"), "new-regular-value");
  assertEquals(diff.subscriptions.size, 0);
});

// ============================================================================
// Array Element Collapse Tests
// ============================================================================

Deno.test("collapseArrayElementUnsets - collapses when all properties of array element are unset", () => {
  const unsetPaths = [
    "/domain/Tags/3/Key",
    "/domain/Tags/3/Value",
  ];

  const existingAttrs = {
    "/domain/Tags/3/Key": "Looking",
    "/domain/Tags/3/Value": "Somebody to love",
  };

  const result = collapseArrayElementUnsets(unsetPaths, existingAttrs);

  assertEquals(result, ["/domain/Tags/3"]);
});

Deno.test("collapseArrayElementUnsets - does NOT collapse when only some properties are unset", () => {
  const unsetPaths = [
    "/domain/Tags/3/Key",
  ];

  const existingAttrs = {
    "/domain/Tags/3/Key": "Looking",
    "/domain/Tags/3/Value": "Somebody to love",
  };

  const result = collapseArrayElementUnsets(unsetPaths, existingAttrs);

  assertEquals(result, ["/domain/Tags/3/Key"]);
});

Deno.test("collapseArrayElementUnsets - handles multiple array elements independently", () => {
  const unsetPaths = [
    "/domain/Tags/0/Key",
    "/domain/Tags/0/Value",
    "/domain/Tags/3/Key",
    "/domain/Tags/3/Value",
  ];

  const existingAttrs = {
    "/domain/Tags/0/Key": "Name",
    "/domain/Tags/0/Value": "demo-load-balancer",
    "/domain/Tags/3/Key": "Looking",
    "/domain/Tags/3/Value": "Somebody to love",
  };

  const result = collapseArrayElementUnsets(unsetPaths, existingAttrs);

  assertEquals(result, ["/domain/Tags/0", "/domain/Tags/3"]);
});

Deno.test("collapseArrayElementUnsets - mixed scenario: some complete, some partial", () => {
  const unsetPaths = [
    "/domain/Tags/0/Key",
    "/domain/Tags/0/Value",
    "/domain/Tags/3/Key", // Only Key, not Value
  ];

  const existingAttrs = {
    "/domain/Tags/0/Key": "Name",
    "/domain/Tags/0/Value": "demo-load-balancer",
    "/domain/Tags/3/Key": "Looking",
    "/domain/Tags/3/Value": "Somebody to love",
  };

  const result = collapseArrayElementUnsets(unsetPaths, existingAttrs);

  // Tags/0 should collapse, Tags/3/Key should remain individual
  assertEquals(result, ["/domain/Tags/0", "/domain/Tags/3/Key"]);
});

Deno.test("collapseArrayElementUnsets - preserves non-array paths", () => {
  const unsetPaths = [
    "/domain/Name",
    "/domain/Tags/3/Key",
    "/domain/Tags/3/Value",
  ];

  const existingAttrs = {
    "/domain/Name": "test",
    "/domain/Tags/3/Key": "Looking",
    "/domain/Tags/3/Value": "Somebody to love",
  };

  const result = collapseArrayElementUnsets(unsetPaths, existingAttrs);

  assertEquals(result, ["/domain/Name", "/domain/Tags/3"]);
});

Deno.test("collapseArrayElementUnsets - handles nested objects within array elements", () => {
  const unsetPaths = [
    "/domain/SecurityGroups/1/GroupId",
    "/domain/SecurityGroups/1/GroupName",
  ];

  const existingAttrs = {
    "/domain/SecurityGroups/1/GroupId": "sg-12345",
    "/domain/SecurityGroups/1/GroupName": "default",
  };

  const result = collapseArrayElementUnsets(unsetPaths, existingAttrs);

  assertEquals(result, ["/domain/SecurityGroups/1"]);
});

Deno.test("collapseArrayElementUnsets - ignores deeply nested properties", () => {
  const unsetPaths = [
    "/domain/Tags/0/Key",
    "/domain/Tags/0/Metadata/Created",
  ];

  const existingAttrs = {
    "/domain/Tags/0/Key": "Name",
    "/domain/Tags/0/Metadata/Created": "2024-01-01",
  };

  const result = collapseArrayElementUnsets(unsetPaths, existingAttrs);

  // Should NOT collapse because Metadata/Created is a nested property
  // (we only count direct children)
  assertEquals(result, [
    "/domain/Tags/0/Key",
    "/domain/Tags/0/Metadata/Created",
  ]);
});

Deno.test("collapseArrayElementUnsets - handles empty array", () => {
  const unsetPaths: string[] = [];
  const existingAttrs = {};

  const result = collapseArrayElementUnsets(unsetPaths, existingAttrs);

  assertEquals(result, []);
});

Deno.test("computeAttributeDiff - full integration: collapses array element unsets", () => {
  const workingSet = {
    "/domain/Name": "test",
  };

  const existing = {
    "/domain/Name": "test",
    "/domain/Tags/3/Key": "Looking",
    "/domain/Tags/3/Value": "Somebody to love",
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.unset.length, 1);
  assertEquals(diff.unset[0], "/domain/Tags/3");
});

Deno.test("computeAttributeDiff - full integration: does NOT collapse partial array element unsets", () => {
  const workingSet = {
    "/domain/Name": "test",
    "/domain/Tags/3/Value": "Somebody to love", // Keep Value
  };

  const existing = {
    "/domain/Name": "test",
    "/domain/Tags/3/Key": "Looking",
    "/domain/Tags/3/Value": "Somebody to love",
  };

  const diff = computeAttributeDiff(workingSet, existing);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.unset.length, 1);
  assertEquals(diff.unset[0], "/domain/Tags/3/Key");
});

Deno.test("attributeDiffToUpdatePayload - formats collapsed array element unset correctly", () => {
  const diff = {
    set: new Map(),
    unset: ["/domain/Tags/3"],
    subscriptions: new Map(),
  };

  const payload = attributeDiffToUpdatePayload(diff);

  assertEquals(payload, {
    "/domain/Tags/3": { "$source": null },
  });
});
