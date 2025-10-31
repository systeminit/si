import { beforeEach, describe, expect, test, vi } from "vitest";
import { AttributePath } from "@/api/sdf/dal/component";

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
