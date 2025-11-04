import { expect, test, vi } from "vitest";
import { mount } from "@vue/test-utils";

// REQUIRED for all testing
import { plugins } from "@/newhotness/testing/index";

// FIXTURES for this test
import { CONTEXT } from "@/newhotness/testing/context1";
import { ChangeSet, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import Explore from "./Explore.vue";

// Mock the api_composables module
vi.mock("./api_composables", () => ({
  routes: {
    CreateChangeSet: "/create-changeset",
  },
  useApi: vi.fn(),
}));

// Mock the router
vi.mock("vue-router", () => ({
  useRouter: () => ({
    currentRoute: {
      value: {
        query: {},
      },
    },
  }),
}));

// Mock the toast composable
vi.mock("@/newhotness/ui_composables/toast", () => ({
  useToast: () => vi.fn(),
}));

// EVERY TEST needs to copypasta this
type HeimdallInner = typeof import("@/store/realtime/heimdall_inner");
vi.mock("@/store/realtime/heimdall", async () => {
  const inner = await vi.importActual<HeimdallInner>(
    "@/store/realtime/heimdall_inner",
  );
  return {
    useMakeKey: () => inner.innerUseMakeKey(CONTEXT.value),
    useMakeArgs: () => inner.innerUseMakeArgs(CONTEXT.value),
  };
});

test("bulkChangeSet.endpoint creates change set successfully", async () => {
  const mockChangeSetData: ChangeSet = {
    id: "test-changeset-id",
    name: "Bulk Edit by Test User",
    status: ChangeSetStatus.Open,
    baseChangeSetId: null,
  };

  const mockApiResponse = {
    data: mockChangeSetData,
  };

  const mockEndpoint = vi.fn().mockResolvedValue({
    post: vi.fn().mockResolvedValue({
      req: mockApiResponse,
    }),
  });

  const mockOk = vi.fn().mockReturnValue(true);
  const mockNavigateToNewChangeSet = vi.fn();

  // Mock useApi to return our mocked functions
  const { useApi } = await import("./api_composables");
  vi.mocked(useApi).mockReturnValue({
    endpoint: mockEndpoint,
    ok: mockOk,
    navigateToNewChangeSet: mockNavigateToNewChangeSet,
    /* eslint-disable @typescript-eslint/no-explicit-any */
  } as any);

  const wrapper = mount(Explore, {
    global: {
      provide: {
        CONTEXT,
      },
      plugins,
    },
  });

  // Access the component's startBulkEdit method and call it
  /* eslint-disable @typescript-eslint/no-explicit-any */
  const component = wrapper.vm as any;
  await component.startBulkEdit();

  // Verify the endpoint was called with correct route
  expect(mockEndpoint).toHaveBeenCalledWith("/create-changeset");

  // Verify the post request was made with correct data
  const endpointCall = mockEndpoint.mock.results[0]?.value;
  expect(endpointCall.post).toHaveBeenCalledWith({
    name: "Bulk Edit by Test User",
  });

  // Verify ok() was called to check response
  expect(mockOk).toHaveBeenCalledWith(mockApiResponse);

  // Verify navigation was called with correct parameters
  expect(mockNavigateToNewChangeSet).toHaveBeenCalledWith({
    name: "new-hotness",
    params: {
      workspacePk: CONTEXT.value.workspacePk,
      changeSetId: "test-changeset-id",
    },
    query: {
      b: "1",
    },
  });
});

test("bulkChangeSet.endpoint handles API error response", async () => {
  const mockApiResponse = {
    error: "Change set creation failed",
  };

  const mockEndpoint = vi.fn().mockResolvedValue({
    post: vi.fn().mockResolvedValue({
      req: mockApiResponse,
    }),
  });

  const mockOk = vi.fn().mockReturnValue(false);
  const mockNavigateToNewChangeSet = vi.fn();

  // Mock useApi to return our mocked functions
  const { useApi } = await import("./api_composables");
  vi.mocked(useApi).mockReturnValue({
    endpoint: mockEndpoint,
    ok: mockOk,
    navigateToNewChangeSet: mockNavigateToNewChangeSet,
  } as any);

  const wrapper = mount(Explore, {
    global: {
      provide: {
        CONTEXT,
      },
      plugins,
    },
  });

  // Access the component's startBulkEdit method and call it
  /* eslint-disable @typescript-eslint/no-explicit-any */
  const component = wrapper.vm as any;
  await component.startBulkEdit();

  // Verify the endpoint was called
  expect(mockEndpoint).toHaveBeenCalledWith("/create-changeset");

  // Verify ok() was called and returned false
  expect(mockOk).toHaveBeenCalledWith(mockApiResponse);

  // Verify navigation was NOT called due to error
  expect(mockNavigateToNewChangeSet).not.toHaveBeenCalled();
});

test("bulkChangeSet.endpoint handles network error", async () => {
  const networkError = new Error("Network request failed");

  const mockEndpoint = vi.fn().mockResolvedValue({
    post: vi.fn().mockRejectedValue(networkError),
  });

  const mockOk = vi.fn();
  const mockNavigateToNewChangeSet = vi.fn();

  // Mock useApi to return our mocked functions
  const { useApi } = await import("./api_composables");
  vi.mocked(useApi).mockReturnValue({
    endpoint: mockEndpoint,
    ok: mockOk,
    navigateToNewChangeSet: mockNavigateToNewChangeSet,
    /* eslint-disable @typescript-eslint/no-explicit-any */
  } as any);

  const wrapper = mount(Explore, {
    global: {
      provide: {
        CONTEXT,
      },
      plugins,
    },
  });

  // Access the component's startBulkEdit method and expect it to throw
  /* eslint-disable @typescript-eslint/no-explicit-any */
  const component = wrapper.vm as any;

  await expect(component.startBulkEdit()).rejects.toThrow(
    "Network request failed",
  );

  // Verify the endpoint was called
  expect(mockEndpoint).toHaveBeenCalledWith("/create-changeset");

  // Verify ok() and navigation were NOT called due to network error
  expect(mockOk).not.toHaveBeenCalled();
  expect(mockNavigateToNewChangeSet).not.toHaveBeenCalled();
});
