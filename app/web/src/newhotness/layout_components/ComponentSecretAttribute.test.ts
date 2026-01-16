import { expect, test, vi } from "vitest";
import { mount } from "@vue/test-utils";

// REQUIRED for all testing
import { plugins } from "@/newhotness/testing/index";

// FIXTURES for this test
import { ATTRIBUTEINPUT, ATTRIBUTE_ERRORS, CONTEXT } from "@/newhotness/testing/context1";
import { component, attributeTree } from "@/newhotness/testing/fixture1";
import { PossibleConnection } from "@/workers/types/entity_kind_types";
import { makeAvTree } from "../logic_composables/attribute_tree";
import ComponentSecretAttribute from "./ComponentSecretAttribute.vue";

// EVERY TEST needs to copypasta this, and add any specific items you need ala getPossibleConnections
type HeimdallInner = typeof import("@/store/realtime/heimdall_inner");
vi.mock("@/store/realtime/heimdall", async () => {
  const inner = await vi.importActual<HeimdallInner>("@/store/realtime/heimdall_inner");
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

test("testing component secret attributes", () => {
  const rootId = Object.keys(attributeTree.treeInfo).find((avId) => {
    const av = attributeTree.treeInfo[avId]!;
    if (!av.parent) return true;
    return false;
  });
  if (!rootId) throw new Error("DATA SETUP FAILURE");

  const tree = makeAvTree(attributeTree, rootId, false);
  const secrets = tree?.children.find((c) => c.prop?.name === "secrets");
  const secret = secrets?.children.pop();
  if (!secret) throw new Error("SECRET SETUP FAILURE");

  const wrapper = mount(ComponentSecretAttribute, {
    global: {
      provide: {
        ATTRIBUTEINPUT,
        CONTEXT,
        ATTRIBUTE_ERRORS,
      },
      plugins,
    },
    props: {
      component,
      attributeTree: secret,
    },
  });

  // there are 2 truncation potentials
  const text = wrapper
    .find("div.cursor-text")
    .findAll("div")
    .map((d) => d.text())
    .join("");
  // ec2 instance should have a secret field that is blank
  expect(text).toBe("");

  // NOTE: can call wrapper.setProps() to change them
});
