import { expect, test, vi } from "vitest";
import { mount } from "@vue/test-utils";

// REQUIRED for all testing
import { plugins } from "@/newhotness/testing/index";

// FIXTURES for this test
import {
  ATTRIBUTEINPUT,
  ATTRIBUTE_ERRORS,
  CONTEXT,
} from "@/newhotness/testing/context1";
import { component, attributeTree } from "@/newhotness/testing/fixture1";
import {
  PossibleConnection,
  ComponentInList,
} from "@/workers/types/entity_kind_types";
import { makeAvTree } from "../logic_composables/attribute_tree";
import AttributeInput from "./AttributeInput.vue";

// EVERY TEST needs to copypasta this, and add any specific items you need ala getPossibleConnections
type HeimdallInner = typeof import("@/store/realtime/heimdall_inner");
vi.mock("@/store/realtime/heimdall", async () => {
  const inner = await vi.importActual<HeimdallInner>(
    "@/store/realtime/heimdall_inner",
  );
  return {
    useMakeKey: () => inner.innerUseMakeKey(CONTEXT.value),
    useMakeArgs: () => inner.innerUseMakeArgs(CONTEXT.value),
    // component needs something here, but not relevant for the test itself
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    getPossibleConnections: async (_args: any) => {
      return [] as PossibleConnection[];
    },
  };
});

/**
 * Tests for AttributeInput readOnly computed property
 *
 * These tests validate that create-only properties are editable when:
 * - The component doesn't exist on HEAD yet (diffStatus === "Added")
 * - The component doesn't have a resource yet (hasResource === false)
 *
 * And that create-only properties are read-only when:
 * - The component exists on HEAD (diffStatus !== "Added" or no diffStatus)
 * - AND the component has a resource (hasResource === true)
 */

test("create-only property is editable when component is Added (not on HEAD) with resource", () => {
  // Find a create-only property in the fixture
  const rootId = Object.keys(attributeTree.treeInfo).find((avId) => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const av = attributeTree.treeInfo[avId]!;
    if (!av.parent) return true;
    return false;
  });
  if (!rootId) throw new Error("DATA SETUP FAILURE");

  const tree = makeAvTree(attributeTree, rootId, false);
  const domain = tree?.children.find((c) => c.prop?.name === "domain");
  const privateIpAddress = domain?.children.find(
    (c) => c.prop?.name === "PrivateIpAddress" && c.prop?.createOnly === true,
  );
  if (!privateIpAddress || !privateIpAddress.prop) {
    throw new Error("CREATE-ONLY PROPERTY SETUP FAILURE");
  }

  // Create a ComponentInList with diffStatus="Added" and hasResource=true
  const componentWithAddedStatus: ComponentInList = {
    ...component,
    diffStatus: "Added", // Component doesn't exist on HEAD
    hasResource: true, // But it has a resource in the change set
  };

  const wrapper = mount(AttributeInput, {
    global: {
      provide: {
        ATTRIBUTEINPUT,
        CONTEXT,
        ATTRIBUTE_ERRORS,
      },
      plugins,
    },
    props: {
      path: privateIpAddress.attributeValue.path,
      value: "",
      kind: privateIpAddress.prop.widgetKind,
      prop: privateIpAddress.prop,
      validation: null,
      component: componentWithAddedStatus,
      displayName: privateIpAddress.prop.name,
      canDelete: false,
      disabled: false,
      externalSources: null,
      isArray: false,
      isMap: false,
      isSecret: false,
      isDefaultSource: false,
      disableInputWindow: false,
      forceReadOnly: false,
      hasSocketConnection: false,
    },
  });

  // Check the readOnly computed property via data-readonly attribute
  const inputDiv = wrapper.find("[data-readonly]");
  expect(inputDiv.exists()).toBe(true);
  expect(inputDiv.attributes("data-readonly")).toBe("false");
});

test("create-only property is read-only when component is Modified (exists on HEAD) with resource", () => {
  const rootId = Object.keys(attributeTree.treeInfo).find((avId) => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const av = attributeTree.treeInfo[avId]!;
    if (!av.parent) return true;
    return false;
  });
  if (!rootId) throw new Error("DATA SETUP FAILURE");

  const tree = makeAvTree(attributeTree, rootId, false);
  const domain = tree?.children.find((c) => c.prop?.name === "domain");
  const privateIpAddress = domain?.children.find(
    (c) => c.prop?.name === "PrivateIpAddress" && c.prop?.createOnly === true,
  );
  if (!privateIpAddress || !privateIpAddress.prop) {
    throw new Error("CREATE-ONLY PROPERTY SETUP FAILURE");
  }

  // Create a ComponentInList with diffStatus="Modified" and hasResource=true
  const componentWithModifiedStatus: ComponentInList = {
    ...component,
    diffStatus: "Modified", // Component exists on HEAD
    hasResource: true,
  };

  const wrapper = mount(AttributeInput, {
    global: {
      provide: {
        ATTRIBUTEINPUT,
        CONTEXT,
        ATTRIBUTE_ERRORS,
      },
      plugins,
    },
    props: {
      path: privateIpAddress.attributeValue.path,
      value: "",
      kind: privateIpAddress.prop.widgetKind,
      prop: privateIpAddress.prop,
      validation: null,
      component: componentWithModifiedStatus,
      displayName: privateIpAddress.prop.name,
      canDelete: false,
      disabled: false,
      externalSources: null,
      isArray: false,
      isMap: false,
      isSecret: false,
      isDefaultSource: false,
      disableInputWindow: false,
      forceReadOnly: false,
      hasSocketConnection: false,
    },
  });

  // Check the readOnly computed property via data-readonly attribute
  const inputDiv = wrapper.find("[data-readonly]");
  expect(inputDiv.exists()).toBe(true);
  expect(inputDiv.attributes("data-readonly")).toBe("true");
});

test("create-only property is read-only when component diffStatus is None (exists on HEAD) with resource", () => {
  const rootId = Object.keys(attributeTree.treeInfo).find((avId) => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const av = attributeTree.treeInfo[avId]!;
    if (!av.parent) return true;
    return false;
  });
  if (!rootId) throw new Error("DATA SETUP FAILURE");

  const tree = makeAvTree(attributeTree, rootId, false);
  const domain = tree?.children.find((c) => c.prop?.name === "domain");
  const privateIpAddress = domain?.children.find(
    (c) => c.prop?.name === "PrivateIpAddress" && c.prop?.createOnly === true,
  );
  if (!privateIpAddress || !privateIpAddress.prop) {
    throw new Error("CREATE-ONLY PROPERTY SETUP FAILURE");
  }

  // Create a ComponentInList with diffStatus="None" and hasResource=true
  const componentWithNoneStatus: ComponentInList = {
    ...component,
    diffStatus: "None", // Component exists on HEAD unchanged
    hasResource: true,
  };

  const wrapper = mount(AttributeInput, {
    global: {
      provide: {
        ATTRIBUTEINPUT,
        CONTEXT,
        ATTRIBUTE_ERRORS,
      },
      plugins,
    },
    props: {
      path: privateIpAddress.attributeValue.path,
      value: "",
      kind: privateIpAddress.prop.widgetKind,
      prop: privateIpAddress.prop,
      validation: null,
      component: componentWithNoneStatus,
      displayName: privateIpAddress.prop.name,
      canDelete: false,
      disabled: false,
      externalSources: null,
      isArray: false,
      isMap: false,
      isSecret: false,
      isDefaultSource: false,
      disableInputWindow: false,
      forceReadOnly: false,
      hasSocketConnection: false,
    },
  });

  // Check the readOnly computed property via data-readonly attribute
  const inputDiv = wrapper.find("[data-readonly]");
  expect(inputDiv.exists()).toBe(true);
  expect(inputDiv.attributes("data-readonly")).toBe("true");
});

test("create-only property is editable when component has no resource (regardless of diffStatus)", () => {
  const rootId = Object.keys(attributeTree.treeInfo).find((avId) => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const av = attributeTree.treeInfo[avId]!;
    if (!av.parent) return true;
    return false;
  });
  if (!rootId) throw new Error("DATA SETUP FAILURE");

  const tree = makeAvTree(attributeTree, rootId, false);
  const domain = tree?.children.find((c) => c.prop?.name === "domain");
  const privateIpAddress = domain?.children.find(
    (c) => c.prop?.name === "PrivateIpAddress" && c.prop?.createOnly === true,
  );
  if (!privateIpAddress || !privateIpAddress.prop) {
    throw new Error("CREATE-ONLY PROPERTY SETUP FAILURE");
  }

  // Create a ComponentInList with diffStatus="Modified" but hasResource=false
  const componentWithoutResource: ComponentInList = {
    ...component,
    diffStatus: "Modified", // Exists on HEAD
    hasResource: false, // But no resource yet
  };

  const wrapper = mount(AttributeInput, {
    global: {
      provide: {
        ATTRIBUTEINPUT,
        CONTEXT,
        ATTRIBUTE_ERRORS,
      },
      plugins,
    },
    props: {
      path: privateIpAddress.attributeValue.path,
      value: "",
      kind: privateIpAddress.prop.widgetKind,
      prop: privateIpAddress.prop,
      validation: null,
      component: componentWithoutResource,
      displayName: privateIpAddress.prop.name,
      canDelete: false,
      disabled: false,
      externalSources: null,
      isArray: false,
      isMap: false,
      isSecret: false,
      isDefaultSource: false,
      disableInputWindow: false,
      forceReadOnly: false,
      hasSocketConnection: false,
    },
  });

  // Check the readOnly computed property via data-readonly attribute
  const inputDiv = wrapper.find("[data-readonly]");
  expect(inputDiv.exists()).toBe(true);
  expect(inputDiv.attributes("data-readonly")).toBe("false");
});

test("non-create-only property is always editable (with resource and exists on HEAD)", () => {
  const rootId = Object.keys(attributeTree.treeInfo).find((avId) => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const av = attributeTree.treeInfo[avId]!;
    if (!av.parent) return true;
    return false;
  });
  if (!rootId) throw new Error("DATA SETUP FAILURE");

  const tree = makeAvTree(attributeTree, rootId, false);
  const domain = tree?.children.find((c) => c.prop?.name === "domain");
  // Find a non-create-only property
  const instanceType = domain?.children.find(
    (c) => c.prop?.name === "InstanceType" && c.prop?.createOnly === false,
  );
  if (!instanceType || !instanceType.prop) {
    throw new Error("NON-CREATE-ONLY PROPERTY SETUP FAILURE");
  }

  // Create a ComponentInList with diffStatus="Modified" and hasResource=true
  const componentWithModifiedStatus: ComponentInList = {
    ...component,
    diffStatus: "Modified", // Exists on HEAD
    hasResource: true,
  };

  const wrapper = mount(AttributeInput, {
    global: {
      provide: {
        ATTRIBUTEINPUT,
        CONTEXT,
        ATTRIBUTE_ERRORS,
      },
      plugins,
    },
    props: {
      path: instanceType.attributeValue.path,
      value: "",
      kind: instanceType.prop.widgetKind,
      prop: instanceType.prop,
      validation: null,
      component: componentWithModifiedStatus,
      displayName: instanceType.prop.name,
      canDelete: false,
      disabled: false,
      externalSources: null,
      isArray: false,
      isMap: false,
      isSecret: false,
      isDefaultSource: false,
      disableInputWindow: false,
      forceReadOnly: false,
      hasSocketConnection: false,
    },
  });

  // Check the readOnly computed property via data-readonly attribute
  const inputDiv = wrapper.find("[data-readonly]");
  expect(inputDiv.exists()).toBe(true);
  expect(inputDiv.attributes("data-readonly")).toBe("false");
});

test("any property is read-only when component toDelete is true", () => {
  const rootId = Object.keys(attributeTree.treeInfo).find((avId) => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const av = attributeTree.treeInfo[avId]!;
    if (!av.parent) return true;
    return false;
  });
  if (!rootId) throw new Error("DATA SETUP FAILURE");

  const tree = makeAvTree(attributeTree, rootId, false);
  const domain = tree?.children.find((c) => c.prop?.name === "domain");
  const instanceType = domain?.children.find(
    (c) => c.prop?.name === "InstanceType",
  );
  if (!instanceType || !instanceType.prop) {
    throw new Error("PROPERTY SETUP FAILURE");
  }

  // Create a ComponentInList with toDelete=true
  const componentToDelete: ComponentInList = {
    ...component,
    toDelete: true,
    hasResource: false,
    diffStatus: "Added",
  };

  const wrapper = mount(AttributeInput, {
    global: {
      provide: {
        ATTRIBUTEINPUT,
        CONTEXT,
        ATTRIBUTE_ERRORS,
      },
      plugins,
    },
    props: {
      path: instanceType.attributeValue.path,
      value: "",
      kind: instanceType.prop.widgetKind,
      prop: instanceType.prop,
      validation: null,
      component: componentToDelete,
      displayName: instanceType.prop.name,
      canDelete: false,
      disabled: false,
      externalSources: null,
      isArray: false,
      isMap: false,
      isSecret: false,
      isDefaultSource: false,
      disableInputWindow: false,
      forceReadOnly: false,
      hasSocketConnection: false,
    },
  });

  // Check the readOnly computed property via data-readonly attribute
  const inputDiv = wrapper.find("[data-readonly]");
  expect(inputDiv.exists()).toBe(true);
  expect(inputDiv.attributes("data-readonly")).toBe("true");
});

test("any property is read-only when forceReadOnly is true", () => {
  const rootId = Object.keys(attributeTree.treeInfo).find((avId) => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const av = attributeTree.treeInfo[avId]!;
    if (!av.parent) return true;
    return false;
  });
  if (!rootId) throw new Error("DATA SETUP FAILURE");

  const tree = makeAvTree(attributeTree, rootId, false);
  const domain = tree?.children.find((c) => c.prop?.name === "domain");
  const instanceType = domain?.children.find(
    (c) => c.prop?.name === "InstanceType",
  );
  if (!instanceType || !instanceType.prop) {
    throw new Error("PROPERTY SETUP FAILURE");
  }

  // Create a ComponentInList with normal state
  const normalComponent: ComponentInList = {
    ...component,
    toDelete: false,
    hasResource: false,
    diffStatus: "Added",
  };

  const wrapper = mount(AttributeInput, {
    global: {
      provide: {
        ATTRIBUTEINPUT,
        CONTEXT,
        ATTRIBUTE_ERRORS,
      },
      plugins,
    },
    props: {
      path: instanceType.attributeValue.path,
      value: "",
      kind: instanceType.prop.widgetKind,
      prop: instanceType.prop,
      validation: null,
      component: normalComponent,
      displayName: instanceType.prop.name,
      canDelete: false,
      disabled: false,
      externalSources: null,
      isArray: false,
      isMap: false,
      isSecret: false,
      isDefaultSource: false,
      disableInputWindow: false,
      forceReadOnly: true, // Force read-only
      hasSocketConnection: false,
    },
  });

  // Check the readOnly computed property via data-readonly attribute
  const inputDiv = wrapper.find("[data-readonly]");
  expect(inputDiv.exists()).toBe(true);
  expect(inputDiv.attributes("data-readonly")).toBe("true");
});
