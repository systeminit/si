import { beforeEach, describe, expect, test, vi } from "vitest";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";

// REQUIRED for all testing
import { CONTEXT } from "@/newhotness/testing/context1";

/**
 * Tests for AttributePanel - JSON Pointer escaping for map keys
 *
 * These tests validate that map keys with special characters (/ and ~) are properly
 * escaped according to RFC 6901 (JSON Pointer specification) before being sent to the backend.
 */

// Track API calls made during tests
let mockApiCalls: Array<{
  route: string;
  method: string;
  payload: Record<string, unknown>;
}>;

// Mock heimdall using the inner pattern like other tests
type HeimdallInner = typeof import("@/store/realtime/heimdall_inner");
vi.mock("@/store/realtime/heimdall", async () => {
  const inner = await vi.importActual<HeimdallInner>(
    "@/store/realtime/heimdall_inner",
  );
  return {
    useMakeKey: () => inner.innerUseMakeKey(CONTEXT.value),
    useMakeArgs: () => inner.innerUseMakeArgs(CONTEXT.value),
    bifrost: vi.fn(),
  };
});

// Mock vue-router
vi.mock("vue-router", () => ({
  useRoute: () => ({
    params: { workspacePk: "test-workspace", changeSetId: "test-changeset" },
  }),
}));

// Mock the api composables
vi.mock("./api_composables", () => ({
  routes: {
    UpdateComponentAttributes: "update-component-attributes",
  },
  useApi: () => ({
    endpoint: vi.fn((route: string, _params?: { id?: string }) => ({
      put: vi.fn(async (payload: Record<string, unknown>) => {
        mockApiCalls.push({
          route,
          method: "PUT",
          payload,
        });
        return {
          req: { status: 200 },
          newChangeSetId: "new-changeset-id",
        };
      }),
    })),
    setWatchFn: vi.fn(),
    ok: vi.fn(() => true),
    navigateToNewChangeSet: vi.fn(),
  }),
  componentTypes: {},
}));

beforeEach(() => {
  vi.clearAllMocks();
  mockApiCalls = [];
});

/**
 * Mock API integration tests
 *
 * These tests simulate the setKey behavior and verify that the mocked API
 * endpoint receives correctly escaped paths in the payload, matching how
 * AttributePanel.vue and AttributePanelBulk.vue construct their API calls.
 */
describe("setKey API integration with escaped keys", () => {
  test("API receives correctly escaped forward slash in map key", async () => {
    // Given: The API composable and escape utility (as used in AttributePanel)
    const { useApi, routes } = await import("./api_composables");
    const { escapeJsonPointerSegment } = await import("./util");
    const api = useApi();

    const componentId = "test-component-123";
    const basePath = "/domain/config";
    const key = "test/paul"; // User enters this key in the UI
    const value = {};

    // When: Simulating setKey behavior - this is exactly what AttributePanel does
    const escapedKey = escapeJsonPointerSegment(key);
    const childPath = `${basePath}/${escapedKey}` as AttributePath;
    const payload = { [childPath]: value };

    const call = api.endpoint(routes.UpdateComponentAttributes, {
      id: componentId,
    });
    await call.put(payload);

    // Then: The mock API should have received the payload with escaped path
    expect(mockApiCalls).toHaveLength(1);
    expect(mockApiCalls[0]?.route).toBe("update-component-attributes");
    expect(mockApiCalls[0]?.method).toBe("PUT");
    expect(mockApiCalls[0]?.payload).toHaveProperty(
      "/domain/config/test~1paul",
    );
    expect(mockApiCalls[0]?.payload["/domain/config/test~1paul"]).toEqual({});
  });

  test("API receives correctly escaped tilde in map key", async () => {
    // Given: The API composable and escape utility
    const { useApi, routes } = await import("./api_composables");
    const { escapeJsonPointerSegment } = await import("./util");
    const api = useApi();

    const componentId = "test-component-456";
    const basePath = "/domain/settings";
    const key = "config~key"; // Key contains tilde
    const value = "";

    // When: Simulating setKey behavior with tilde in key
    const escapedKey = escapeJsonPointerSegment(key);
    const childPath = `${basePath}/${escapedKey}` as AttributePath;
    const payload = { [childPath]: value };

    const call = api.endpoint(routes.UpdateComponentAttributes, {
      id: componentId,
    });
    await call.put(payload);

    // Then: The tilde should be escaped as ~0 in the API payload
    expect(mockApiCalls).toHaveLength(1);
    expect(mockApiCalls[0]?.payload).toHaveProperty(
      "/domain/settings/config~0key",
    );
    expect(mockApiCalls[0]?.payload["/domain/settings/config~0key"]).toBe("");
  });

  test("API receives correctly escaped path with both forward slash and tilde", async () => {
    // Given: The API composable and escape utility
    const { useApi, routes } = await import("./api_composables");
    const { escapeJsonPointerSegment } = await import("./util");
    const api = useApi();

    const componentId = "test-component-789";
    const basePath = "/domain/IamPolicies";
    const key = "policy/~admin"; // Key contains both special characters
    const value = { role: "admin" };

    // When: Simulating setKey behavior with both special characters
    const escapedKey = escapeJsonPointerSegment(key);
    const childPath = `${basePath}/${escapedKey}` as AttributePath;
    const payload = { [childPath]: value };

    const call = api.endpoint(routes.UpdateComponentAttributes, {
      id: componentId,
    });
    await call.put(payload);

    // Then: Both characters should be properly escaped (/ → ~1, ~ → ~0)
    expect(mockApiCalls).toHaveLength(1);
    expect(mockApiCalls[0]?.payload).toHaveProperty(
      "/domain/IamPolicies/policy~1~0admin",
    );
    expect(
      mockApiCalls[0]?.payload["/domain/IamPolicies/policy~1~0admin"],
    ).toEqual({ role: "admin" });
  });

  test("API receives correctly escaped real-world AWS ARN with forward slash", async () => {
    // Given: A realistic scenario with an AWS ARN containing forward slash
    const { useApi, routes } = await import("./api_composables");
    const { escapeJsonPointerSegment } = await import("./util");
    const api = useApi();

    const componentId = "test-component-aws";
    const basePath = "/domain/IamRoles";
    const key = "arn:aws:iam::123456789012/role-name"; // Real AWS ARN format
    const value = { arn: key };

    // When: Simulating setKey behavior with AWS ARN
    const escapedKey = escapeJsonPointerSegment(key);
    const childPath = `${basePath}/${escapedKey}` as AttributePath;
    const payload = { [childPath]: value };

    const call = api.endpoint(routes.UpdateComponentAttributes, {
      id: componentId,
    });
    await call.put(payload);

    // Then: The forward slash in the ARN should be escaped
    expect(mockApiCalls).toHaveLength(1);
    expect(mockApiCalls[0]?.payload).toHaveProperty(
      "/domain/IamRoles/arn:aws:iam::123456789012~1role-name",
    );
    expect(
      mockApiCalls[0]?.payload[
        "/domain/IamRoles/arn:aws:iam::123456789012~1role-name"
      ],
    ).toEqual({ arn: key });
  });

  test("API receives multiple escaped forward slashes in path-like keys", async () => {
    // Given: A path-like key with multiple forward slashes
    const { useApi, routes } = await import("./api_composables");
    const { escapeJsonPointerSegment } = await import("./util");
    const api = useApi();

    const componentId = "test-component-path";
    const basePath = "/domain/FilePaths";
    const key = "path/to/resource"; // Multiple slashes
    const value = { location: key };

    // When: Simulating setKey behavior with multiple slashes
    const escapedKey = escapeJsonPointerSegment(key);
    const childPath = `${basePath}/${escapedKey}` as AttributePath;
    const payload = { [childPath]: value };

    const call = api.endpoint(routes.UpdateComponentAttributes, {
      id: componentId,
    });
    await call.put(payload);

    // Then: All forward slashes should be escaped
    expect(mockApiCalls).toHaveLength(1);
    expect(mockApiCalls[0]?.payload).toHaveProperty(
      "/domain/FilePaths/path~1to~1resource",
    );
    expect(
      mockApiCalls[0]?.payload["/domain/FilePaths/path~1to~1resource"],
    ).toEqual({ location: key });
  });

  test("API receives normal keys without modification", async () => {
    // Given: A normal key without special characters
    const { useApi, routes } = await import("./api_composables");
    const { escapeJsonPointerSegment } = await import("./util");
    const api = useApi();

    const componentId = "test-component-normal";
    const basePath = "/domain/config";
    const key = "normalKey123"; // No special characters
    const value = { setting: "value" };

    // When: Simulating setKey behavior with normal key
    const escapedKey = escapeJsonPointerSegment(key);
    const childPath = `${basePath}/${escapedKey}` as AttributePath;
    const payload = { [childPath]: value };

    const call = api.endpoint(routes.UpdateComponentAttributes, {
      id: componentId,
    });
    await call.put(payload);

    // Then: The key should remain unchanged in the API payload
    expect(mockApiCalls).toHaveLength(1);
    expect(mockApiCalls[0]?.payload).toHaveProperty(
      "/domain/config/normalKey123",
    );
    expect(mockApiCalls[0]?.payload["/domain/config/normalKey123"]).toEqual({
      setting: "value",
    });
  });
});

/**
 * Tests for clearing input fields - API receives correct { $source: null } format
 *
 * When users clear an input field in the AttributePanel, the UI must send
 * { $source: null } to properly unset the value and trigger attribute functions
 * to re-run with the default/schema variant prototype.
 *
 * These tests validate the fix for a bug where clearing input fields sent plain null
 * instead of { $source: null }, which caused attribute functions not to re-execute.
 */
describe("API integration with clearing input fields", () => {
  test("API receives { $source: null } when user clears a string input", async () => {
    // Given: The API composable and makeSavePayload function (as used in AttributePanel)
    const { useApi, routes } = await import("./api_composables");
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");
    const api = useApi();

    const componentId = "test-component-clear";
    const path = "/domain/image" as AttributePath;
    const value = ""; // User cleared the input

    // When: Simulating save behavior with cleared input - this is what AttributePanel does
    const payload = makeSavePayload(path, value, PropKind.String);
    const call = api.endpoint(routes.UpdateComponentAttributes, {
      id: componentId,
    });
    await call.put(payload);

    // Then: The mock API should have received { $source: null }
    expect(mockApiCalls).toHaveLength(1);
    expect(mockApiCalls[0]?.route).toBe("update-component-attributes");
    expect(mockApiCalls[0]?.method).toBe("PUT");
    expect(mockApiCalls[0]?.payload).toEqual({
      "/domain/image": { $source: null },
    });
  });

  test("API receives { $source: null } when user clears an integer input", async () => {
    // Given: The API composable and makeSavePayload function
    const { useApi, routes } = await import("./api_composables");
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");
    const api = useApi();

    const componentId = "test-component-clear-int";
    const path = "/domain/count" as AttributePath;
    const value = ""; // User cleared the integer input

    // When: Simulating save behavior with cleared integer input
    const payload = makeSavePayload(path, value, PropKind.Integer);
    const call = api.endpoint(routes.UpdateComponentAttributes, {
      id: componentId,
    });
    await call.put(payload);

    // Then: The mock API should have received { $source: null }
    expect(mockApiCalls).toHaveLength(1);
    expect(mockApiCalls[0]?.payload).toEqual({
      "/domain/count": { $source: null },
    });
  });

  test("API receives actual value when user enters text in string input", async () => {
    // Given: The API composable and makeSavePayload function
    const { useApi, routes } = await import("./api_composables");
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");
    const api = useApi();

    const componentId = "test-component-value";
    const path = "/domain/image" as AttributePath;
    const value = "my-image:v1"; // User entered a value

    // When: Simulating save behavior with actual value
    const payload = makeSavePayload(path, value, PropKind.String);
    const call = api.endpoint(routes.UpdateComponentAttributes, {
      id: componentId,
    });
    await call.put(payload);

    // Then: The mock API should have received the actual value
    expect(mockApiCalls).toHaveLength(1);
    expect(mockApiCalls[0]?.payload).toEqual({
      "/domain/image": "my-image:v1",
    });
  });

  test("API receives coerced integer when user enters number in integer input", async () => {
    // Given: The API composable and makeSavePayload function
    const { useApi, routes } = await import("./api_composables");
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");
    const api = useApi();

    const componentId = "test-component-int";
    const path = "/domain/count" as AttributePath;
    const value = "42"; // User entered an integer

    // When: Simulating save behavior with integer value
    const payload = makeSavePayload(path, value, PropKind.Integer);
    const call = api.endpoint(routes.UpdateComponentAttributes, {
      id: componentId,
    });
    await call.put(payload);

    // Then: The mock API should have received the coerced integer
    expect(mockApiCalls).toHaveLength(1);
    expect(mockApiCalls[0]?.payload).toEqual({
      "/domain/count": 42,
    });
  });

  test("API receives subscription format when connecting components", async () => {
    // Given: The API composable and makeSavePayload function
    const { useApi, routes } = await import("./api_composables");
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");
    const api = useApi();

    const componentId = "test-component-subscription";
    const path = "/domain/output" as AttributePath;
    const value = "/domain/input"; // Path to subscribe to
    const connectingComponentId = "source-component-123" as ComponentId;

    // When: Simulating subscription creation
    const payload = makeSavePayload(
      path,
      value,
      PropKind.String,
      connectingComponentId,
    );
    const call = api.endpoint(routes.UpdateComponentAttributes, {
      id: componentId,
    });
    await call.put(payload);

    // Then: The mock API should have received subscription format
    expect(mockApiCalls).toHaveLength(1);
    expect(mockApiCalls[0]?.payload).toEqual({
      "/domain/output": {
        $source: {
          component: "source-component-123",
          path: "/domain/input",
        },
      },
    });
  });
});

/**
 * Unit tests for makeSavePayload function
 *
 * These tests verify the payload structure returned by makeSavePayload
 * in isolation, without the full API integration.
 */
describe("makeSavePayload unit tests", () => {
  test("makeSavePayload returns { $source: null } for empty string value", async () => {
    // Given: The makeSavePayload function
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");

    // When: User clears an input field (value becomes empty string)
    const payload = makeSavePayload(
      "/domain/image" as AttributePath,
      "", // Empty string from cleared input
      PropKind.String,
    );

    // Then: Payload should contain { $source: null } to properly unset the value
    expect(payload).toEqual({
      "/domain/image": { $source: null },
    });
  });

  test("makeSavePayload returns string value when value is not empty", async () => {
    // Given: The makeSavePayload function
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");

    // When: User enters a string value
    const payload = makeSavePayload(
      "/domain/image" as AttributePath,
      "my-image:v1",
      PropKind.String,
    );

    // Then: Payload should contain the string value
    expect(payload).toEqual({
      "/domain/image": "my-image:v1",
    });
  });

  test("makeSavePayload returns coerced integer for integer prop", async () => {
    // Given: The makeSavePayload function
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");

    // When: User enters an integer value
    const payload = makeSavePayload(
      "/domain/count" as AttributePath,
      "42",
      PropKind.Integer,
    );

    // Then: Payload should contain the coerced integer
    expect(payload).toEqual({
      "/domain/count": 42,
    });
  });

  test("makeSavePayload returns { $source: null } for cleared integer prop", async () => {
    // Given: The makeSavePayload function
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");

    // When: User clears an integer input field
    const payload = makeSavePayload(
      "/domain/count" as AttributePath,
      "", // Empty string from cleared input
      PropKind.Integer,
    );

    // Then: Payload should contain { $source: null } regardless of prop kind
    expect(payload).toEqual({
      "/domain/count": { $source: null },
    });
  });

  test("makeSavePayload returns coerced boolean for boolean prop", async () => {
    // Given: The makeSavePayload function
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");

    // When: User enters a boolean value
    const payload = makeSavePayload(
      "/domain/enabled" as AttributePath,
      "true",
      PropKind.Boolean,
    );

    // Then: Payload should contain the coerced boolean
    expect(payload).toEqual({
      "/domain/enabled": true,
    });
  });

  test("makeSavePayload returns coerced float for float prop", async () => {
    // Given: The makeSavePayload function
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");

    // When: User enters a float value
    const payload = makeSavePayload(
      "/domain/price" as AttributePath,
      "19.99",
      PropKind.Float,
    );

    // Then: Payload should contain the coerced float
    expect(payload).toEqual({
      "/domain/price": 19.99,
    });
  });

  test("makeSavePayload returns subscription format when connecting component", async () => {
    // Given: The makeSavePayload function
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");

    // When: User creates a subscription connection
    const payload = makeSavePayload(
      "/domain/output" as AttributePath,
      "/domain/input",
      PropKind.String,
      "component-123" as ComponentId,
    );

    // Then: Payload should contain { $source: { component, path } }
    expect(payload).toEqual({
      "/domain/output": {
        $source: {
          component: "component-123",
          path: "/domain/input",
        },
      },
    });
  });

  test("makeSavePayload returns subscription format even with empty path", async () => {
    // Given: The makeSavePayload function
    const { makeSavePayload } = await import(
      "./logic_composables/attribute_tree"
    );
    const { PropKind } = await import("@/api/sdf/dal/prop");

    // When: Creating a subscription with empty path (edge case)
    const payload = makeSavePayload(
      "/domain/output" as AttributePath,
      "",
      PropKind.String,
      "component-123" as ComponentId,
    );

    // Then: Should use subscription format, not $source: null
    expect(payload).toEqual({
      "/domain/output": {
        $source: {
          component: "component-123",
          path: "",
        },
      },
    });
  });
});
