import { beforeEach, describe, expect, test, vi } from "vitest";
import { computed, ref } from "vue";
import { flushPromises, mount } from "@vue/test-utils";

import { plugins } from "@/newhotness/testing/index";
import { CONTEXT } from "@/newhotness/testing/context1";

const WORKSPACE_PK = "01HRFEV0S23R1G23RP75QQDCA7";
const CHANGE_SET_ID = "01K45ZAY3PQPJ457V65KNCC66F";

const mockWorkspaceMetadata = {
  defaultChangeSetId: "01JYPTEC5JM3T1Y4ECEPT9560J",
  changeSets: [
    {
      id: "01K45ZAY3PQPJ457V65KNCC66F",
      name: "test",
      status: "Open",
      baseChangeSetId: "01JYPTEC5JM3T1Y4ECEPT9560J",
      createdAt: "2025-09-02T19:44:20.609624Z",
      updatedAt: "2025-09-08T21:11:45.779873Z",
      workspaceId: "01HRFEV0S23R1G23RP75QQDCA7",
    },
  ],
  approvers: [],
};

type HeimdallInner = typeof import("@/store/realtime/heimdall_inner");
vi.mock("@/store/realtime/heimdall", async () => {
  const inner = await vi.importActual<HeimdallInner>(
    "@/store/realtime/heimdall_inner",
  );
  return {
    useMakeKey: () => inner.innerUseMakeKey(CONTEXT.value),
    useMakeArgs: () => inner.innerUseMakeArgs(CONTEXT.value),
    useMakeArgsForHead: () => inner.innerUseMakeArgs(CONTEXT.value),
    useMakeKeyForHead: () => () => computed(() => ["key"]),
    bifrost: vi.fn().mockResolvedValue(null),
    bifrostExists: vi.fn().mockReturnValue(false),
    initCompleted: ref(true),
    wsConnections: ref({ "01HRFEV0S23R1G23RP75QQDCA7": true }),
    indexFailures: new Set<string>(),
    indexTouches: new Map<string, number>(),
    muspelheimStatuses: ref({}),
    ChangeSetRetrievalError: class extends Error {},
    init: vi.fn().mockResolvedValue(undefined),
    showInterest: vi.fn().mockResolvedValue(undefined),
    bifrostReconnect: vi.fn().mockResolvedValue(undefined),
    muspelheim: vi.fn().mockResolvedValue(undefined),
    niflheim: vi.fn().mockResolvedValue(undefined),
    registerBearerToken: vi.fn().mockResolvedValue(undefined),
    syncAtoms: vi.fn().mockResolvedValue(undefined),
    linkNewChangeset: vi.fn().mockResolvedValue(undefined),
    prune: vi.fn().mockResolvedValue(undefined),
    getOutgoingConnectionsCounts: vi.fn().mockResolvedValue({}),
    getComponentDetails: vi.fn().mockResolvedValue({}),
    getSchemaMembers: vi.fn().mockResolvedValue([]),
    changeSetExists: vi.fn().mockResolvedValue(true),
  };
});

vi.mock("vue-router", async (importOriginal) => {
  const actual = await importOriginal<typeof import("vue-router")>();
  const routeParams = {
    workspacePk: "01HRFEV0S23R1G23RP75QQDCA7",
    changeSetId: "01K45ZAY3PQPJ457V65KNCC66F",
  };
  return {
    ...actual,
    useRoute: () => ({
      name: "new-hotness",
      path: "/n/01HRFEV0S23R1G23RP75QQDCA7/01K45ZAY3PQPJ457V65KNCC66F/h",
      params: routeParams,
      query: {},
    }),
    useRouter: () => ({
      push: vi.fn(),
      replace: vi.fn(),
      currentRoute: ref({ name: "new-hotness", params: routeParams }),
    }),
  };
});

vi.mock("./api_composables", async (importOriginal) => {
  const original = await importOriginal<typeof import("./api_composables")>();
  return {
    ...original,
    useApi: () => ({
      endpoint: vi.fn(() => ({
        get: vi.fn().mockResolvedValue({ data: mockWorkspaceMetadata }),
        post: vi.fn().mockResolvedValue({ req: { status: 200 } }),
        put: vi.fn().mockResolvedValue({ req: { status: 200 } }),
      })),
      setWatchFn: vi.fn(),
      bifrosting: ref(false),
      inFlight: ref(false),
      ok: vi.fn(() => true),
      navigateToNewChangeSet: vi.fn(),
    }),
  };
});

vi.mock("@tanstack/vue-query", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@tanstack/vue-query")>();
  return {
    ...actual,
    useQuery: (options: { enabled?: { value: boolean } }) => {
      // Accessing enabled.value triggers Vue's reactivity chain.
      // If there's a circular dependency, this throws a TDZ error.
      if (
        options.enabled &&
        typeof options.enabled === "object" &&
        "value" in options.enabled
      ) {
        const _ = options.enabled.value;
      }
      return {
        data: ref(mockWorkspaceMetadata),
        isLoading: ref(false),
        isFetched: ref(true),
      };
    },
    useQueryClient: () => ({
      setQueryData: vi.fn(),
      invalidateQueries: vi.fn(),
      setDefaultOptions: vi.fn(),
    }),
  };
});

vi.mock("./logic_composables/navigation_stack", () => ({
  reset: vi.fn(),
  push: vi.fn(),
  prevPage: vi.fn(),
  query: {},
}));

vi.mock("@/utils/posthog", () => ({
  posthog: { capture: vi.fn() },
}));

const mountOptions = {
  global: {
    plugins,
    stubs: {
      ConnectionLine: true,
      OnboardingModal: true,
      teleport: true,
    },
  },
  props: {
    workspacePk: WORKSPACE_PK,
    changeSetId: CHANGE_SET_ID,
  },
};

function assertNotCircularReferenceError(error: Error, context: string): void {
  if (
    error.message.includes("Cannot access") &&
    error.message.includes("before initialization")
  ) {
    expect.fail(
      `CIRCULAR REFERENCE DETECTED ${context}: "${error.message}"\n` +
        `Fix: useChangeSets should use ApiContext (not full Context) to break the cycle.`,
    );
  }
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("Workspace.vue - Circular Reference Prevention", () => {
  test("mounting Workspace should not throw circular reference error", async () => {
    const Workspace = await import("./Workspace.vue");
    let wrapper: ReturnType<typeof mount> | undefined;

    try {
      wrapper = mount(Workspace.default, mountOptions);
      await flushPromises();
    } catch (e) {
      assertNotCircularReferenceError(e as Error, "on mount");
      throw e;
    }

    expect(wrapper).toBeDefined();
    wrapper?.unmount();
  });
});
