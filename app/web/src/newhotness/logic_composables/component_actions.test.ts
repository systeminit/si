import { beforeEach, expect, test, vi } from "vitest";
import { computed, ref } from "vue";
import { ActionKind } from "@/api/sdf/dal/action";
import {
  ActionPrototypeView,
  BifrostComponent,
  ActionPrototypeViewList,
  BifrostActionViewList,
} from "@/workers/types/entity_kind_types";

// REQUIRED for all testing
import { CONTEXT } from "@/newhotness/testing/context1";

/**
 * Tests for useComponentActions composable - refreshEnabled logic
 *
 * These tests validate the refreshEnabled computed property behavior.
 * The refreshEnabled property controls visibility of the refresh button on the ComponentDetails
 * resource panel.
 */

// Track query responses for each test
let mockQueryResponses: Map<string, unknown>;

// Mock heimdall using the inner pattern like other tests
type HeimdallInner = typeof import("@/store/realtime/heimdall_inner");
vi.mock("@/store/realtime/heimdall", async () => {
  const inner = await vi.importActual<HeimdallInner>(
    "@/store/realtime/heimdall_inner",
  );
  return {
    useMakeKey: () => inner.innerUseMakeKey(CONTEXT.value),
    useMakeKeyForHead: () => (kind: string, id?: string) => {
      const ctx = CONTEXT.value;
      return computed(() => [
        ctx.workspacePk.value,
        ctx.headChangeSetId.value,
        kind,
        id ?? ctx.workspacePk.value,
      ]);
    },
    useMakeArgs: () => inner.innerUseMakeArgs(CONTEXT.value),
    useMakeArgsForHead: () => (kind: string, id?: string) => {
      const ctx = CONTEXT.value;
      return {
        workspaceId: ctx.workspacePk.value,
        changeSetId: ctx.headChangeSetId.value,
        kind,
        id: id ?? ctx.workspacePk.value,
      };
    },
    bifrost: vi.fn(),
    bifrostExists: vi.fn(),
  };
});

// Mock vue-router
vi.mock("vue-router", () => ({
  useRoute: () => ({
    params: { workspacePk: "test-workspace", changeSetId: "test-changeset" },
  }),
}));

// Mock the api composables
vi.mock("../api_composables", () => ({
  routes: {
    RefreshAction: "refresh-action",
    ActionCancel: "action-cancel",
    ActionAdd: "action-add",
  },
  useApi: () => ({
    endpoint: vi.fn(() => ({
      put: vi.fn(),
      post: vi.fn(),
    })),
    setWatchFn: vi.fn(),
    bifrosting: ref(false),
    ok: vi.fn(() => true),
    navigateToNewChangeSet: vi.fn(),
  }),
}));

// Mock @tanstack/vue-query
vi.mock("@tanstack/vue-query", () => ({
  useQuery: (options: {
    queryKey: { value: unknown[] };
    queryFn: () => Promise<unknown>;
    enabled?: { value: boolean };
  }) => {
    // Create a simple string key from the query key array
    const keyArray = options.queryKey.value;
    const simpleKey = keyArray
      .map((item) => {
        // Unwrap refs/computed
        const unwrapped =
          item !== null &&
          typeof item === "object" &&
          "value" in item &&
          "effect" in item
            ? (item as { value: unknown }).value
            : item;

        if (typeof unwrapped === "string") return unwrapped;
        if (typeof unwrapped === "object" && unwrapped !== null) {
          // Check if it's a WeakReference with an id property
          if ("id" in unwrapped && typeof unwrapped.id === "string") {
            return unwrapped.id;
          }
          return JSON.stringify(unwrapped);
        }
        return String(unwrapped);
      })
      .join("|");

    const data = ref(mockQueryResponses.get(simpleKey) ?? null);
    return {
      data,
      isLoading: ref(false),
      isFetched: ref(true),
    };
  },
}));

// Mock context to allow overriding onHead per test
let mockOnHead = false;
vi.mock("./context", () => ({
  useContext: () => {
    const baseContext = CONTEXT.value;
    return {
      ...baseContext,
      onHead: computed(() => mockOnHead),
    };
  },
}));

beforeEach(() => {
  vi.clearAllMocks();
  mockQueryResponses = new Map();
  mockOnHead = false;
});

/**
 * The refreshEnabled computed property should return true when ALL of the following are true:
 * 1. refreshActionPrototype.value exists (the schema variant has a refresh action defined)
 * 2. component.value?.hasResource is true (the component has a resource)
 * 3. EITHER:
 *    - ctx.onHead.value is true (viewing component on HEAD), OR
 *    - componentExistsOnHead.value is true (component was applied to HEAD, viewing from change set)
 */

test("refreshEnabled returns true when on HEAD with resource and refresh action", async () => {
  // Given: User is on HEAD with a component that has a resource and refresh action
  mockOnHead = true;

  const mockComponent = ref<BifrostComponent>({
    id: "test-component",
    hasResource: true,
    schemaVariantId: { id: "test-variant" },
  } as BifrostComponent);

  const mockActionPrototypeViewList: ActionPrototypeViewList = {
    id: "test-variant",
    actionPrototypes: [
      {
        id: "refresh-prototype",
        kind: ActionKind.Refresh,
        name: "Refresh",
      } as ActionPrototypeView,
    ],
  };

  const mockActionViewList: BifrostActionViewList = {
    id: CONTEXT.value.changeSetId.value,
    actions: [],
  };

  // Set up query responses with full keys including workspace and changeset IDs from CONTEXT
  const ctx = CONTEXT.value;
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.changeSetId.value}|ActionPrototypeViewList|test-variant`,
    mockActionPrototypeViewList,
  );
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.changeSetId.value}|ActionViewList|${ctx.workspacePk.value}`,
    mockActionViewList,
  );
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.headChangeSetId.value}|ComponentInList|test-component`,
    true,
  );

  const { useComponentActions } = await import("./component_actions");
  const { refreshEnabled } = useComponentActions(mockComponent);

  // Then: refreshEnabled should be true
  expect(refreshEnabled.value).toBe(true);
});

test("refreshEnabled returns true when in change set with component that exists on HEAD", async () => {
  // Given: User is in a change set, component exists on HEAD with resource and refresh action
  mockOnHead = false;

  const mockComponent = ref<BifrostComponent>({
    id: "test-component",
    hasResource: true,
    schemaVariantId: { id: "test-variant" },
  } as BifrostComponent);

  const mockActionPrototypeViewList: ActionPrototypeViewList = {
    id: "test-variant",
    actionPrototypes: [
      {
        id: "refresh-prototype",
        kind: ActionKind.Refresh,
        name: "Refresh",
      } as ActionPrototypeView,
    ],
  };

  const mockActionViewList: BifrostActionViewList = {
    id: CONTEXT.value.changeSetId.value,
    actions: [],
  };

  // Set up query responses with full keys including workspace and changeset IDs from CONTEXT
  const ctx = CONTEXT.value;
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.changeSetId.value}|ActionPrototypeViewList|test-variant`,
    mockActionPrototypeViewList,
  );
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.changeSetId.value}|ActionViewList|${ctx.workspacePk.value}`,
    mockActionViewList,
  );
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.headChangeSetId.value}|ComponentInList|test-component`,
    true, // componentExistsOnHead = true
  );

  const { useComponentActions } = await import("./component_actions");
  const { refreshEnabled } = useComponentActions(mockComponent);

  // Then: refreshEnabled should be true
  expect(refreshEnabled.value).toBe(true);
});

test("refreshEnabled returns false when not on HEAD and component doesn't exist on HEAD", async () => {
  // Given: User is in a change set, component doesn't exist on HEAD yet
  mockOnHead = false;

  const mockComponent = ref<BifrostComponent>({
    id: "test-component",
    hasResource: true,
    schemaVariantId: { id: "test-variant" },
  } as BifrostComponent);

  const mockActionPrototypeViewList: ActionPrototypeViewList = {
    id: "test-variant",
    actionPrototypes: [
      {
        id: "refresh-prototype",
        kind: ActionKind.Refresh,
        name: "Refresh",
      } as ActionPrototypeView,
    ],
  };

  const mockActionViewList: BifrostActionViewList = {
    id: CONTEXT.value.changeSetId.value,
    actions: [],
  };

  // Set up query responses with full keys including workspace and changeset IDs from CONTEXT
  const ctx = CONTEXT.value;
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.changeSetId.value}|ActionPrototypeViewList|test-variant`,
    mockActionPrototypeViewList,
  );
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.changeSetId.value}|ActionViewList|${ctx.workspacePk.value}`,
    mockActionViewList,
  );
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.headChangeSetId.value}|ComponentInList|test-component`,
    false, // componentExistsOnHead = false
  );

  const { useComponentActions } = await import("./component_actions");
  const { refreshEnabled } = useComponentActions(mockComponent);

  // Then: refreshEnabled should be false (can't refresh a resource that hasn't been applied to HEAD yet)
  expect(refreshEnabled.value).toBe(false);
});

test("refreshEnabled returns false when component has no resource", async () => {
  // Given: User is on HEAD but component has no resource
  mockOnHead = true;

  const mockComponent = ref<BifrostComponent>({
    id: "test-component",
    hasResource: false, // No resource!
    schemaVariantId: { id: "test-variant" },
  } as BifrostComponent);

  const mockActionPrototypeViewList: ActionPrototypeViewList = {
    id: "test-variant",
    actionPrototypes: [
      {
        id: "refresh-prototype",
        kind: ActionKind.Refresh,
        name: "Refresh",
      } as ActionPrototypeView,
    ],
  };

  const mockActionViewList: BifrostActionViewList = {
    id: CONTEXT.value.changeSetId.value,
    actions: [],
  };

  // Set up query responses with full keys including workspace and changeset IDs from CONTEXT
  const ctx = CONTEXT.value;
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.changeSetId.value}|ActionPrototypeViewList|test-variant`,
    mockActionPrototypeViewList,
  );
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.changeSetId.value}|ActionViewList|${ctx.workspacePk.value}`,
    mockActionViewList,
  );

  const { useComponentActions } = await import("./component_actions");
  const { refreshEnabled } = useComponentActions(mockComponent);

  // Then: refreshEnabled should be false (can't refresh a non-existent resource)
  expect(refreshEnabled.value).toBe(false);
});

test("refreshEnabled returns false when no refresh action prototype exists", async () => {
  // Given: User is on HEAD but schema variant has no refresh action
  mockOnHead = true;

  const mockComponent = ref<BifrostComponent>({
    id: "test-component",
    hasResource: true,
    schemaVariantId: { id: "test-variant" },
  } as BifrostComponent);

  const mockActionPrototypeViewList: ActionPrototypeViewList = {
    id: "test-variant",
    actionPrototypes: [], // No refresh action!
  };

  const mockActionViewList: BifrostActionViewList = {
    id: CONTEXT.value.changeSetId.value,
    actions: [],
  };

  // Set up query responses with full keys including workspace and changeset IDs from CONTEXT
  const ctx = CONTEXT.value;
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.changeSetId.value}|ActionPrototypeViewList|test-variant`,
    mockActionPrototypeViewList,
  );
  mockQueryResponses.set(
    `${ctx.workspacePk.value}|${ctx.changeSetId.value}|ActionViewList|${ctx.workspacePk.value}`,
    mockActionViewList,
  );

  const { useComponentActions } = await import("./component_actions");
  const { refreshEnabled } = useComponentActions(mockComponent);

  // Then: refreshEnabled should be false (schema doesn't support refresh)
  expect(refreshEnabled.value).toBe(false);
});
