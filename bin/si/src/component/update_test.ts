/**
 * Tests for component update module
 *
 * @module
 */

import { assertEquals } from "@std/assert";
import { computeAttributeDiff } from "../template/attribute_diff.ts";
import { Context } from "../context.ts";

// Initialize context for testing
Context.init({ verbose: 0, noColor: true });

Deno.test("computeAttributeDiff - detects new attributes", () => {
  const desired = {
    "/domain/region": "us-west-2",
    "/domain/instanceType": "t3.micro",
  };
  const current = {
    "/domain/region": "us-east-1",
  };

  const diff = computeAttributeDiff(desired, current);

  assertEquals(diff.set.size, 2);
  assertEquals(diff.set.get("/domain/region"), "us-west-2");
  assertEquals(diff.set.get("/domain/instanceType"), "t3.micro");
  assertEquals(diff.unset.length, 0);
});

Deno.test("computeAttributeDiff - detects removed attributes", () => {
  const desired = {
    "/domain/region": "us-west-2",
  };
  const current = {
    "/domain/region": "us-west-2",
    "/domain/instanceType": "t3.micro",
    "/domain/tags": { env: "prod" },
  };

  const diff = computeAttributeDiff(desired, current);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.unset.length, 2);
  assertEquals(diff.unset.includes("/domain/instanceType"), true);
  assertEquals(diff.unset.includes("/domain/tags"), true);
});

Deno.test("computeAttributeDiff - detects changed values", () => {
  const desired = {
    "/domain/region": "us-west-2",
    "/domain/instanceType": "t3.large",
  };
  const current = {
    "/domain/region": "us-east-1",
    "/domain/instanceType": "t3.micro",
  };

  const diff = computeAttributeDiff(desired, current);

  assertEquals(diff.set.size, 2);
  assertEquals(diff.set.get("/domain/region"), "us-west-2");
  assertEquals(diff.set.get("/domain/instanceType"), "t3.large");
  assertEquals(diff.unset.length, 0);
});

Deno.test("computeAttributeDiff - no changes when identical", () => {
  const desired = {
    "/domain/region": "us-west-2",
    "/domain/instanceType": "t3.micro",
  };
  const current = {
    "/domain/region": "us-west-2",
    "/domain/instanceType": "t3.micro",
  };

  const diff = computeAttributeDiff(desired, current);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.unset.length, 0);
  assertEquals(diff.subscriptions.size, 0);
});

Deno.test("computeAttributeDiff - handles subscriptions", () => {
  const desired = {
    "/domain/connection": {
      "$source": {
        component: "comp-123",
        path: "/domain/output",
      },
    },
  };
  const current = {
    "/domain/connection": {
      "$source": {
        component: "comp-456",
        path: "/domain/output",
      },
    },
  };

  const diff = computeAttributeDiff(desired, current);

  assertEquals(diff.set.size, 0);
  assertEquals(diff.subscriptions.size, 1);
  assertEquals(
    diff.subscriptions.get("/domain/connection")?.component,
    "comp-123",
  );
});

Deno.test("computeAttributeDiff - handles nested objects", () => {
  const desired = {
    "/domain/config": {
      timeout: 30,
      retries: 3,
    },
  };
  const current = {
    "/domain/config": {
      timeout: 60,
      retries: 3,
    },
  };

  const diff = computeAttributeDiff(desired, current);

  assertEquals(diff.set.size, 1);
  assertEquals(diff.set.get("/domain/config"), { timeout: 30, retries: 3 });
});

Deno.test("computeAttributeDiff - handles arrays", () => {
  const desired = {
    "/domain/items": ["a", "b", "c"],
  };
  const current = {
    "/domain/items": ["a", "b"],
  };

  const diff = computeAttributeDiff(desired, current);

  assertEquals(diff.set.size, 1);
  assertEquals(diff.set.get("/domain/items"), ["a", "b", "c"]);
});

Deno.test("computeAttributeDiff - preserves /si attributes", () => {
  const desired = {
    "/si/name": "my-component",
    "/si/color": "#ff0000",
    "/domain/region": "us-west-2",
  };
  const current = {
    "/si/name": "old-name",
    "/si/color": "#ff0000",
    "/domain/region": "us-west-2",
  };

  const diff = computeAttributeDiff(desired, current);

  assertEquals(diff.set.size, 1);
  assertEquals(diff.set.get("/si/name"), "my-component");
  assertEquals(diff.unset.length, 0);
});

Deno.test("computeAttributeDiff - handles null values", () => {
  const desired = {
    "/domain/optionalField": null,
  };
  const current = {
    "/domain/optionalField": "some-value",
  };

  const diff = computeAttributeDiff(desired, current);

  assertEquals(diff.set.size, 1);
  assertEquals(diff.set.get("/domain/optionalField"), null);
});
