import { beforeEach, describe, expect, test, vi } from "vitest";
import { ref } from "vue";
import { mount } from "@vue/test-utils";
import { ActionKind, ActionState } from "@/api/sdf/dal/action";
import { ActionProposedView } from "./types";
import ActionQueueList from "./ActionQueueList.vue";

/**
 * Tests for ActionQueueList - Duplicate action detection logic
 *
 * These tests validate that child actions are correctly hidden from the top-level
 * list when they have parents that are Queued or OnHold, preventing duplicates
 * in the UI (since ActionQueueListItem recursively renders children).
 */

// Mock components to avoid deep rendering issues
vi.mock("./EmptyState.vue", () => ({
  default: {
    name: "EmptyState",
    template: "<div>EmptyState</div>",
  },
}));

vi.mock("./ActionQueueListItem.vue", () => ({
  default: {
    name: "ActionQueueListItem",
    props: [
      "action",
      "actionsById",
      "child",
      "noInteraction",
      "actionChildren",
    ],
    template: '<div class="action-item">{{ action.name }}</div>',
  },
}));

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

beforeEach(() => {
  vi.clearAllMocks();
});

describe("ActionQueueList - hasDisplayedParent logic preventing duplicates", () => {
  test("child action with Queued parent is hidden from top-level Queued section", () => {
    // Given: A parent action that is Queued and a child action that is also Queued
    const parentAction: ActionProposedView = {
      id: "parent-1",
      prototypeId: "proto-1",
      componentId: "comp-1",
      name: "Parent Create Action",
      description: "Create parent resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-1",
      state: ActionState.Queued,
      myDependencies: ["child-1"], // Backend includes all descendants
      dependentOn: [],
      holdStatusInfluencedBy: [],
      componentSchemaName: "ParentSchema",
      componentName: "parent-component",
    };

    const childAction: ActionProposedView = {
      id: "child-1",
      prototypeId: "proto-2",
      componentId: "comp-2",
      name: "Child Create Action",
      description: "Create child resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-1",
      state: ActionState.Queued,
      myDependencies: [],
      dependentOn: ["parent-1"], // Depends on parent
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

    // Then: Only parent should appear in the Queued section (child will be nested under parent)
    const actionItems = wrapper.findAllComponents({
      name: "ActionQueueListItem",
    });

    expect(actionItems.length).toBe(1);
    expect(actionItems[0]?.props("action").id).toBe("parent-1");
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

    expect(actionItems.length).toBe(1);
    expect(actionItems[0]?.props("action").id).toBe("parent-2");
  });

  // NOTE WE ARE DUPING SOME IN  ActionQueueListItem.test.ts
  // IF WE CHANGE THESE, CHANGE THOSE TOO
  test("child action is hidden when parent is OnHold and child is Queued (different states)", () => {
    // Given: A parent action that is OnHold and a child action that is Queued
    const parentAction: ActionProposedView = {
      id: "parent-3",
      prototypeId: "proto-5",
      componentId: "comp-5",
      name: "Parent Update Action",
      description: "Update parent resource",
      kind: ActionKind.Update,
      originatingChangeSetId: "cs-3",
      state: ActionState.OnHold,
      myDependencies: ["child-3"],
      dependentOn: [],
      holdStatusInfluencedBy: [],
      componentSchemaName: "ParentSchema",
      componentName: "parent-component",
    };

    const childAction: ActionProposedView = {
      id: "child-3",
      prototypeId: "proto-6",
      componentId: "comp-6",
      name: "Child Update Action",
      description: "Update child resource",
      kind: ActionKind.Update,
      originatingChangeSetId: "cs-3",
      state: ActionState.Queued, // Different state from parent!
      myDependencies: [],
      dependentOn: ["parent-3"],
      holdStatusInfluencedBy: ["parent-3"],
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

    // Then: Only parent should appear at top level in OnHold section
    const actionItems = wrapper.findAllComponents({
      name: "ActionQueueListItem",
    });

    expect(actionItems.length).toBe(1);
    expect(actionItems[0]?.props("action").id).toBe("parent-3");
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

  test("deeply nested actions with transitive dependencies only show root at top level", () => {
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

    expect(actionItems.length).toBe(1);
    expect(actionItems[0]?.props("action").id).toBe("grandparent-1");
  });

  test("standalone action with no dependencies appears at top level", () => {
    // Given: An action with no parent dependencies
    const standaloneAction: ActionProposedView = {
      id: "standalone-1",
      prototypeId: "proto-12",
      componentId: "comp-12",
      name: "Standalone Create Action",
      description: "Create resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-6",
      state: ActionState.Queued,
      myDependencies: [],
      dependentOn: [], // No parents
      holdStatusInfluencedBy: [],
      componentSchemaName: "StandaloneSchema",
      componentName: "standalone-component",
    };

    const actionViewList = ref([standaloneAction]);
    const highlightedActionIds = ref(new Set<string>());

    // When: Rendering the ActionQueueList
    const wrapper = mount(ActionQueueList, {
      props: {
        actionViewList: actionViewList.value,
        highlightedActionIds: highlightedActionIds.value,
      },
    });

    // Then: The standalone action should appear at top level
    const actionItems = wrapper.findAllComponents({
      name: "ActionQueueListItem",
    });

    expect(actionItems.length).toBe(1);
    expect(actionItems[0]?.props("action").id).toBe("standalone-1");
  });

  test("multiple independent parent-child pairs each show only their parent at top level", () => {
    // Given: Two independent parent-child pairs
    const parent1: ActionProposedView = {
      id: "parent-6",
      prototypeId: "proto-13",
      componentId: "comp-13",
      name: "Parent 1 Create Action",
      description: "Create parent 1 resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-7",
      state: ActionState.Queued,
      myDependencies: ["child-6"],
      dependentOn: [],
      holdStatusInfluencedBy: [],
      componentSchemaName: "ParentSchema1",
      componentName: "parent-1-component",
    };

    const child1: ActionProposedView = {
      id: "child-6",
      prototypeId: "proto-14",
      componentId: "comp-14",
      name: "Child 1 Create Action",
      description: "Create child 1 resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-7",
      state: ActionState.Queued,
      myDependencies: [],
      dependentOn: ["parent-6"],
      holdStatusInfluencedBy: [],
      componentSchemaName: "ChildSchema1",
      componentName: "child-1-component",
    };

    const parent2: ActionProposedView = {
      id: "parent-7",
      prototypeId: "proto-15",
      componentId: "comp-15",
      name: "Parent 2 Create Action",
      description: "Create parent 2 resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-7",
      state: ActionState.OnHold,
      myDependencies: ["child-7"],
      dependentOn: [],
      holdStatusInfluencedBy: [],
      componentSchemaName: "ParentSchema2",
      componentName: "parent-2-component",
    };

    const child2: ActionProposedView = {
      id: "child-7",
      prototypeId: "proto-16",
      componentId: "comp-16",
      name: "Child 2 Create Action",
      description: "Create child 2 resource",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-7",
      state: ActionState.OnHold,
      myDependencies: [],
      dependentOn: ["parent-7"],
      holdStatusInfluencedBy: ["parent-7"],
      componentSchemaName: "ChildSchema2",
      componentName: "child-2-component",
    };

    const actionViewList = ref([parent1, child1, parent2, child2]);
    const highlightedActionIds = ref(new Set<string>());

    // When: Rendering the ActionQueueList
    const wrapper = mount(ActionQueueList, {
      props: {
        actionViewList: actionViewList.value,
        highlightedActionIds: highlightedActionIds.value,
      },
    });

    // Then: Only the two parents should appear at top level
    const actionItems = wrapper.findAllComponents({
      name: "ActionQueueListItem",
    });

    expect(actionItems.length).toBe(2);
    const actionIds = actionItems.map((item) => item.props("action").id);
    expect(actionIds).toContain("parent-6");
    expect(actionIds).toContain("parent-7");
    expect(actionIds).not.toContain("child-6");
    expect(actionIds).not.toContain("child-7");
  });

  test("four-level dependency chain with transitive myDependencies shows only root", () => {
    // Given: The actual bug scenario from the issue - 4 components with subscription chain
    // Component 4 -> Component 3 -> Component 2 -> Component 1
    // Backend populates myDependencies with ALL descendants (transitive closure)
    const comp4Action: ActionProposedView = {
      id: "comp-4",
      prototypeId: "proto-17",
      componentId: "comp-17",
      name: "Component 4 Create",
      description: "Create",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-8",
      state: ActionState.Queued,
      myDependencies: ["comp-3", "comp-2", "comp-1"], // ALL descendants!
      dependentOn: [],
      holdStatusInfluencedBy: [],
      componentSchemaName: "TestResourceActions",
      componentName: "4si-6147",
    };

    const comp3Action: ActionProposedView = {
      id: "comp-3",
      prototypeId: "proto-18",
      componentId: "comp-18",
      name: "Component 3 Create",
      description: "Create",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-8",
      state: ActionState.Queued,
      myDependencies: ["comp-2", "comp-1"], // All descendants below this
      dependentOn: ["comp-4"], // Direct parent only
      holdStatusInfluencedBy: [],
      componentSchemaName: "TestResourceActions",
      componentName: "32si-6147",
    };

    const comp2Action: ActionProposedView = {
      id: "comp-2",
      prototypeId: "proto-19",
      componentId: "comp-19",
      name: "Component 2 Create",
      description: "Create",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-8",
      state: ActionState.Queued,
      myDependencies: ["comp-1"], // All descendants below this
      dependentOn: ["comp-3"], // Direct parent only
      holdStatusInfluencedBy: [],
      componentSchemaName: "TestResourceActions",
      componentName: "2si-6147",
    };

    const comp1Action: ActionProposedView = {
      id: "comp-1",
      prototypeId: "proto-20",
      componentId: "comp-20",
      name: "Component 1 Create",
      description: "Create",
      kind: ActionKind.Create,
      originatingChangeSetId: "cs-8",
      state: ActionState.Queued,
      myDependencies: [], // Leaf node
      dependentOn: ["comp-2"], // Direct parent only
      holdStatusInfluencedBy: [],
      componentSchemaName: "TestResourceActions",
      componentName: "1si-6147",
    };

    const actionViewList = ref([
      comp4Action,
      comp3Action,
      comp2Action,
      comp1Action,
    ]);
    const highlightedActionIds = ref(new Set<string>());

    // When: Rendering the ActionQueueList
    const wrapper = mount(ActionQueueList, {
      props: {
        actionViewList: actionViewList.value,
        highlightedActionIds: highlightedActionIds.value,
      },
    });

    // Then: Only the root action (comp-4) should appear at top level
    // Without the fix in ActionQueueListItem, comp-4 would recursively render ALL its
    // myDependencies (comp-3, comp-2, comp-1), then comp-3 would render (comp-2, comp-1),
    // then comp-2 would render comp-1, causing multiplicative duplication
    const actionItems = wrapper.findAllComponents({
      name: "ActionQueueListItem",
    });

    expect(actionItems.length).toBe(1);
    expect(actionItems[0]?.props("action").id).toBe("comp-4");
  });
});
