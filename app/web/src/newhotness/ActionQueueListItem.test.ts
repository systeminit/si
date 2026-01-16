import { beforeEach, describe, expect, test, vi } from "vitest";
import { ref } from "vue";
import { mount } from "@vue/test-utils";
import { ActionKind, ActionState } from "@/api/sdf/dal/action";
import { ActionProposedView } from "./types";
import ActionQueueList from "./ActionQueueList.vue";

/**
 * NOTE: I broke this out into a separate file because once the `ActionQueueListItem`
 * gets mocked, even per test, it doesn't want to get unmocked for this test.
 */

vi.mock("./api_composables", () => ({
  routes: {
    ActionRetry: "action-retry",
  },
  useApi: () => ({
    endpoint: vi.fn(() => ({
      put: vi.fn(),
    })),
  }),
}));

vi.mock("si/vue-lib/design-system/DropdownMenu.vue", () => ({
  default: {
    name: "DropdownMenu",
    template: '<div class="dropdownmenu">Dropdown</div>',
  },
}));

vi.mock("vue-router", async (importOriginal) => {
  const actual = await importOriginal();
  return {
    ...(actual as object),
    useRoute: () => ({
      params: { workspacePk: "test-workspace", changeSetId: "test-changeset" },
    }),
  };
});

beforeEach(() => {
  vi.clearAllMocks();
});

describe("Test that the ActionQueueListItem correctly displays children", () => {
  test("Circular dependency between two components and a third that depends on one of them. (from prod)", () => {
    /**
     * We are not *actively* supporting circular dependencies, e.g. they will
     * not display circularly——but we do at least want to not die if we get data
     * shaped like this (be defensive!!!)
     *
     * NOTE: the current decision is that each action will be displayed once
     * one of the circular dependency becomes "the parent", and the 3rd action
     * slides underneath that one.
     *
     * But an equally valid decision could be that 4 actions show up, because
     * we've changed how nesting works... so if this test case fails because 3 != 4
     * its possible that it should just be changed to 4.
     */

    const actionViewList = ref([
      {
        id: "01KCME2ZTRRZYPBDNDB1EZJDFN",
        prototypeId: "01KCM5WWD3V79WH3HT9BQEYE6K",
        componentId: "01KCM5WWDEEC6HRS55XKPXVS03",
        componentSchemaName: "Microsoft.Resources/resourceGroups",
        componentName: "hello-world-rg",
        name: "Destroy",
        description: "Delete Resource Group",
        kind: "Destroy",
        state: "Queued",
        originatingChangeSetId: "01KCME0RBFNA6Q0YNHTRPWWDHP",
        myDependencies: ["01KCME2ZTRRZYPBDNDB1EZJDFN", "01KCME6FWAH3GH1WY2KGA70Q9M", "01KCME70ANXAXDDSNZ19CAZY3G"],
        dependentOn: ["01KCME6FWAH3GH1WY2KGA70Q9M"],
        holdStatusInfluencedBy: [],
      },
      {
        id: "01KCME6FWAH3GH1WY2KGA70Q9M",
        prototypeId: "01KCM62JGYXTFJG50RYDYX93NR",
        componentId: "01KCME6C82XYSMSRBETGXFJ8GV",
        componentSchemaName: "Microsoft.Web/sites",
        componentName: "hello-world-func-20251216",
        name: "Create",
        description: null,
        kind: "Create",
        state: "Queued",
        originatingChangeSetId: "01KCME0RBFNA6Q0YNHTRPWWDHP",
        myDependencies: ["01KCME2ZTRRZYPBDNDB1EZJDFN", "01KCME6FWAH3GH1WY2KGA70Q9M", "01KCME70ANXAXDDSNZ19CAZY3G"],
        dependentOn: ["01KCME2ZTRRZYPBDNDB1EZJDFN"],
        holdStatusInfluencedBy: [],
      },
      {
        id: "01KCME70ANXAXDDSNZ19CAZY3G",
        prototypeId: "01KCM89B8B0G5G03FG35T4S1VS",
        componentId: "01KCME6ZFTZT25BZ9JRK4WHXGE",
        componentSchemaName: "Microsoft.Authorization/roleAssignments",
        componentName: "func-app-storage-blob-data-owner",
        name: "Create",
        description: null,
        kind: "Create",
        state: "Queued",
        originatingChangeSetId: "01KCME0RBFNA6Q0YNHTRPWWDHP",
        myDependencies: [],
        dependentOn: ["01KCME6FWAH3GH1WY2KGA70Q9M"],
        holdStatusInfluencedBy: [],
      },
    ] as ActionProposedView[]);
    const highlightedActionIds = ref(new Set<string>());

    const wrapper = mount(ActionQueueList, {
      props: {
        actionViewList: actionViewList.value,
        highlightedActionIds: highlightedActionIds.value,
      },
    });

    // Then: Only grandparent should appear at top level
    // ActionQueueListItem will recursively render children, and with the fix,
    const actionItems = wrapper.findAllComponents({
      name: "ActionQueueListItem",
    });

    expect(actionItems.length).toBe(3);
  });

  // NOTE THESE ARE FROM ActionQueueList.test.ts
  // IF WE CHANGE THOSE, CHANGE THESE TOO
  test("deeply nested actions with transitive dependencies", () => {
    // Given: A three-level dependency chain where myDependencies contains ALL descendants
    // This simulates the actual backend behavior that caused the bug
    const grandparentAction: ActionProposedView = {
      id: "grandparent-1",
      prototypeId: "proto-9",
      componentId: "comp-9",
      name: "Grandparent Create Action",
      description: "Create grandparent resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-5",
      state: ActionState.Queued,
      myDependencies: ["parent-5", "child-5"], // Backend includes ALL descendants!
      dependentOn: [],
      holdStatusInfluencedBy: [],
      componentSchemaName: "GrandparentSchema",
      componentName: "grandparent-component",
    };

    const parentAction: ActionProposedView = {
      id: "parent-5",
      prototypeId: "proto-10",
      componentId: "comp-10",
      name: "Parent Create Action",
      description: "Create parent resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-5",
      state: ActionState.Queued,
      myDependencies: ["child-5"], // Backend includes all descendants
      dependentOn: ["grandparent-1"], // Only lists direct parent
      holdStatusInfluencedBy: [],
      componentSchemaName: "ParentSchema",
      componentName: "parent-component",
    };

    const childAction: ActionProposedView = {
      id: "child-5",
      prototypeId: "proto-11",
      componentId: "comp-11",
      name: "Child Create Action",
      description: "Create child resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-5",
      state: ActionState.Queued,
      myDependencies: [],
      dependentOn: ["parent-5"], // Only lists direct parent
      holdStatusInfluencedBy: [],
      componentSchemaName: "ChildSchema",
      componentName: "child-component",
    };

    const actionViewList = ref([grandparentAction, parentAction, childAction]);
    const highlightedActionIds = ref(new Set<string>());

    // When: Rendering the ActionQueueList
    const wrapper = mount(ActionQueueList, {
      props: {
        actionViewList: actionViewList.value,
        highlightedActionIds: highlightedActionIds.value,
      },
    });

    // Then: Only grandparent should appear at top level
    // ActionQueueListItem will recursively render children, and with the fix,
    // it will only render direct children (not all descendants from myDependencies)
    const actionItems = wrapper.findAllComponents({
      name: "ActionQueueListItem",
    });

    expect(actionItems.length).toBe(3);
  });

  test("child action is hidden even when parent is Queued and child is OnHold (different states)", () => {
    // Given: A parent action that is Queued and a child action that is OnHold
    // This was part of the bug - the old code required exact state match
    const parentAction: ActionProposedView = {
      id: "parent-2",
      prototypeId: "proto-3",
      componentId: "comp-3",
      name: "Parent Create Action",
      description: "Create parent resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-2",
      state: ActionState.Queued,
      myDependencies: ["child-2"],
      dependentOn: [],
      holdStatusInfluencedBy: [],
      componentSchemaName: "ParentSchema",
      componentName: "parent-component",
    };

    const childAction: ActionProposedView = {
      id: "child-2",
      prototypeId: "proto-4",
      componentId: "comp-4",
      name: "Child Create Action",
      description: "Create child resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-2",
      state: ActionState.OnHold, // Different state from parent!
      myDependencies: [],
      dependentOn: ["parent-2"],
      holdStatusInfluencedBy: ["parent-2"],
      componentSchemaName: "ChildSchema",
      componentName: "child-component",
    };

    const actionViewList = ref([parentAction, childAction]);
    const highlightedActionIds = ref(new Set<string>());

    // When: Rendering the ActionQueueList
    const wrapper = mount(ActionQueueList, {
      props: {
        actionViewList: actionViewList.value,
        highlightedActionIds: highlightedActionIds.value,
      },
    });

    // Then: Only parent should appear at top level, child should not appear in OnHold section
    const actionItems = wrapper.findAllComponents({
      name: "ActionQueueListItem",
    });

    expect(actionItems.length).toBe(2);
  });

  test("child action appears at top level when parent is Running (not Queued/OnHold)", () => {
    // Given: A parent action that is Running and a child action that is Queued
    const parentAction: ActionProposedView = {
      id: "parent-4",
      prototypeId: "proto-7",
      componentId: "comp-7",
      name: "Parent Create Action",
      description: "Create parent resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-4",
      state: ActionState.Running, // Running parents don't render children
      myDependencies: ["child-4"],
      dependentOn: [],
      holdStatusInfluencedBy: [],
      componentSchemaName: "ParentSchema",
      componentName: "parent-component",
    };

    const childAction: ActionProposedView = {
      id: "child-4",
      prototypeId: "proto-8",
      componentId: "comp-8",
      name: "Child Create Action",
      description: "Create child resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-4",
      state: ActionState.Queued,
      myDependencies: [],
      dependentOn: ["parent-4"],
      holdStatusInfluencedBy: [],
      componentSchemaName: "ChildSchema",
      componentName: "child-component",
    };

    const actionViewList = ref([parentAction, childAction]);
    const highlightedActionIds = ref(new Set<string>());

    // When: Rendering the ActionQueueList
    const wrapper = mount(ActionQueueList, {
      props: {
        actionViewList: actionViewList.value,
        highlightedActionIds: highlightedActionIds.value,
      },
    });

    // Then: Both parent and child should appear at top level
    const actionItems = wrapper.findAllComponents({
      name: "ActionQueueListItem",
    });

    expect(actionItems.length).toBe(2);
    const actionIds = actionItems.map((item) => item.props("action").id);
    expect(actionIds).toContain("parent-4");
    expect(actionIds).toContain("child-4");
  });

  test("child action appears at top level when parent is Failed (not Queued/OnHold)", () => {
    // Given: A parent action that is Running and a child action that is Queued
    const parentAction: ActionProposedView = {
      id: "parent-4",
      prototypeId: "proto-7",
      componentId: "comp-7",
      name: "Parent Create Action",
      description: "Create parent resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-4",
      state: ActionState.Failed, // Running parents don't render children
      myDependencies: ["child-4"],
      dependentOn: [],
      holdStatusInfluencedBy: [],
      componentSchemaName: "ParentSchema",
      componentName: "parent-component",
    };

    const childAction: ActionProposedView = {
      id: "child-4",
      prototypeId: "proto-8",
      componentId: "comp-8",
      name: "Child Create Action",
      description: "Create child resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-4",
      state: ActionState.Queued,
      myDependencies: [],
      dependentOn: ["parent-4"],
      holdStatusInfluencedBy: [],
      componentSchemaName: "ChildSchema",
      componentName: "child-component",
    };

    const actionViewList = ref([parentAction, childAction]);
    const highlightedActionIds = ref(new Set<string>());

    // When: Rendering the ActionQueueList
    const wrapper = mount(ActionQueueList, {
      props: {
        actionViewList: actionViewList.value,
        highlightedActionIds: highlightedActionIds.value,
      },
    });

    // Then: Both parent and child should appear at top level
    const actionItems = wrapper.findAllComponents({
      name: "ActionQueueListItem",
    });

    expect(actionItems.length).toBe(2);
    const actionIds = actionItems.map((item) => item.props("action").id);
    expect(actionIds).toContain("parent-4");
    expect(actionIds).toContain("child-4");
  });
});
