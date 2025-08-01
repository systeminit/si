<template>
  <div>
    <div
      :class="
        clsx(
          'flex flex-row items-center gap-xs px-xs mb-xs',
          themeClasses('bg-neutral-300', 'bg-neutral-600'),
        )
      "
    >
      <IconButton
        tooltip="Close (Esc)"
        tooltipPlacement="top"
        class="border-0 mr-2em"
        icon="x"
        size="sm"
        iconIdleTone="shade"
        iconTone="shade"
        @click="() => emit('close')"
      />
      <h1 class="text-md font-bold my-xs">Bulk Editing</h1>
    </div>
    <ul class="flex flex-row gap-xs">
      <li v-for="component in selectedComponents" :key="component.id">
        <ComponentCard :component="component" />
      </li>
    </ul>
    <DelayedLoader v-if="treesPending" :size="'full'" />
    <div
      v-else-if="commonTree.domain || commonTree.secrets"
      :class="
        clsx(
          'mt-sm p-xs',
          // these styles are from CollapsingFlexItem
          'border overflow-hidden',
          themeClasses(
            'border-neutral-300 bg-white',
            'border-neutral-600 bg-neutral-800',
          ),
        )
      "
      class=""
    >
      <AttributeChildLayout v-if="commonTree.domain">
        <template #header><span class="text-sm">domain</span></template>
        <ComponentAttribute
          v-for="child in commonTree.domain.children"
          :key="child.id"
          :component="componentMap[child.componentId]!"
          :attributeTree="child"
          @save="save"
          @add="add"
          @set-key="setKey"
          @remove-subscription="removeSubscription"
          @delete="remove"
        />
      </AttributeChildLayout>
      <AttributeChildLayout
        v-if="commonTree.secrets && commonTree.secrets.children.length > 0"
      >
        <template #header><span class="text-sm">secrets</span></template>
        <ComponentSecretAttribute
          v-for="secret in commonTree.secrets.children"
          :key="secret.id"
          :component="componentMap[secret.componentId]!"
          :attributeTree="secret"
        />
      </AttributeChildLayout>
    </div>
    <p v-else class="italic text-center mt-md">
      The selected components do not share any common attributes.
    </p>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { useQueries } from "@tanstack/vue-query";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { themeClasses, IconButton } from "@si/vue-lib/design-system";
import { useToast } from "vue-toastification";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  AttributeTree,
  ComponentInList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import { keyEmitter } from "@/newhotness/logic_composables/emitters";
import ComponentCard from "@/newhotness/ComponentCard.vue";
import { nonNullable } from "@/utils/typescriptLinter";
import ComponentAttribute, {
  NewChildValue,
} from "@/newhotness/layout_components/ComponentAttribute.vue";
import ComponentSecretAttribute from "@/newhotness/layout_components/ComponentSecretAttribute.vue";
import AttributeChildLayout from "@/newhotness/layout_components/AttributeChildLayout.vue";
import DelayedLoader from "@/newhotness/layout_components/DelayedLoader.vue";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";
import { PropKind } from "@/api/sdf/dal/prop";
import {
  arrayAttrTreeIntoTree,
  AttrTree,
  makeAvTree,
  makeSavePayload,
} from "../logic_composables/attribute_tree";
import { componentTypes, ok, routes, UseApi, useApi } from "../api_composables";
import { useContext } from "../logic_composables/context";

const ctx = useContext();

const props = defineProps<{
  selectedComponents: ComponentInList[];
}>();

const componentMap = computed(() =>
  props.selectedComponents.reduce((obj, component) => {
    obj[component.id] = component;
    return obj;
  }, {} as Record<string, ComponentInList>),
);
const componentIds = computed(() => Object.keys(componentMap.value));

const makeKey = useMakeKey();
const makeArgs = useMakeArgs();

// first get all of the relevant AV trees
const queries = computed(() =>
  componentIds.value.map((id) => {
    return {
      queryKey: makeKey(EntityKind.AttributeTree, id),
      queryFn: async () =>
        await bifrost<AttributeTree>(makeArgs(EntityKind.AttributeTree, id)),
    };
  }),
);
const avTrees = useQueries({
  queries,
});

const treesPending = computed(() =>
  avTrees.value.some((t) => t.status === "pending"),
);

// now turn them all into the AttrTree type, starting at domain
type Trees = Array<{
  root: AttrTree | undefined;
  domain: AttrTree | undefined;
  secrets: AttrTree | undefined;
}>;
const trees = computed<Trees>(() => {
  return avTrees.value
    .map((t) => t.data)
    .filter(nonNullable)
    .map((t) => {
      const rootId = Object.keys(t.treeInfo).find((avId) => {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        const av = t.treeInfo[avId]!;
        if (!av.parent) return true;
        return false;
      });
      if (!rootId) return null;

      const tree = makeAvTree(t, rootId, false);
      const domain = tree.children.find((c) => c.prop?.name === "domain");
      const secrets = tree.children.find((c) => c.prop?.name === "secrets");
      return { domain, secrets, root: tree };
    })
    .filter(nonNullable);
});

// now filter them to the common AV paths present in all trees
// the *actual* AVs will be from the very last component that has them
// because of how we are using `pathMap`
const commonTree = computed<{
  domain: AttrTree | undefined;
  secrets: AttrTree | undefined;
}>(() => {
  const pathMap: Record<string, AttrTree> = {};
  const first = trees.value[0];
  if (!first) return { domain: undefined, secrets: undefined };
  const firstPaths = new Set<string>();
  const paths = new Set<string>();
  const firstsChildren = [first.domain, first.secrets];

  // get all the paths from the first component
  while (firstsChildren.length > 0) {
    const attr = firstsChildren.shift();
    if (!attr) continue;
    pathMap[attr.attributeValue.path] = attr;
    firstPaths.add(attr.attributeValue.path);
    firstsChildren.push(...attr.children);
  }

  // find paths in the subsequent components that match the first
  trees.value.slice(1).forEach((componentAttrs) => {
    const walking = [componentAttrs.domain, componentAttrs.secrets];
    while (walking.length > 0) {
      const attr = walking.shift();
      if (!attr) continue;
      if (!firstPaths.has(attr.attributeValue.path)) continue;
      pathMap[attr.attributeValue.path] = attr;
      paths.add(attr.attributeValue.path);
      walking.push(...attr.children);
    }
  });

  // now remove any of the paths from the first component, that isn't in the rest
  const commonPaths = [...firstPaths].filter((p) => paths.has(p));
  const matches = commonPaths
    .map((path) => pathMap[path])
    .filter(nonNullable)
    .map((t) => {
      // clear out the AVs and externalSources
      // because we don't want them showing up in the form
      const m = {
        ...t,
        attributeValue: {
          ...t.attributeValue,
          value: null,
          externalSources: undefined,
        },
      };
      return m;
    });
  const map = matches.reduce((obj, attr) => {
    obj[attr.id] = attr;
    return obj;
  }, {} as Record<string, AttrTree>);

  const matchesAsTree = arrayAttrTreeIntoTree(matches, map);

  const domain = Object.values(matchesAsTree).find(
    (t) => t.prop?.name === "domain",
  );
  const secrets = Object.values(matchesAsTree).find(
    (t) => t.prop?.name === "secrets",
  );
  return {
    domain: matchesAsTree[domain?.id || ""],
    secrets: matchesAsTree[secrets?.id || ""],
  };
});

type ApiVal = {
  value: string;
  propKind: PropKind;
  connectingComponentId?: ComponentId;
};

const saving = ref(0);

const createCalls = () =>
  componentIds.value.map((componentId) => {
    const saveApi = useApi(ctx);
    const call = saveApi.endpoint<{ success: boolean }>(
      routes.UpdateComponentAttributes,
      { id: componentId },
    );
    return call;
  });

const createPayload = (path: AttributePath, vals: ApiVal) =>
  makeSavePayload(path, vals.value, vals.propKind, vals.connectingComponentId);

const add = async (
  _: UseApi,
  attributeTree: AttrTree,
  value: NewChildValue,
) => {
  if (ctx.onHead.value) throw new Error("Must be on a change set");

  const apis = createCalls();

  const calls = apis.map(async (call) => {
    const appendPath =
      `${attributeTree.attributeValue.path}/-` as AttributePath;
    const payload = {
      [appendPath]: value,
    };
    return call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  });

  saving.value = calls.length;
  const resps = await Promise.all(calls);
  saving.value = 0;
  if (!resps.every((r) => ok(r.req))) {
    const errs = resps.map((r) => [r.req, r.req.status, r.req.request]);
    toast(`API Error: ${errs}`);
  }
};

const setKey = async (
  attributeTree: AttrTree,
  key: string,
  value: NewChildValue,
) => {
  if (ctx.onHead.value) throw new Error("Must be on a change set");

  const apis = createCalls();

  const calls = apis.map(async (call) => {
    const appendPath =
      `${attributeTree.attributeValue.path}/${key}` as AttributePath;
    const payload = {
      [appendPath]: value,
    };
    return call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  });

  saving.value = calls.length;
  const resps = await Promise.all(calls);
  saving.value = 0;
  if (!resps.every((r) => ok(r.req))) {
    const errs = resps.map((r) => [r.req, r.req.status, r.req.request]);
    toast(`API Error: ${errs}`);
  }
};

const toast = useToast();

const save = async (
  path: AttributePath,
  value: string,
  propKind: PropKind,
  connectingComponentId?: ComponentId,
) => {
  // TODO force change set if on HEAD when starting
  if (ctx.onHead.value) throw new Error("Must be on a change set");

  // one API call per component
  const apis = createCalls();

  const calls = apis.map(async (call) => {
    const payload = createPayload(path, {
      value,
      propKind,
      connectingComponentId,
    });

    return await call.put<componentTypes.UpdateComponentAttributesArgs>(
      payload,
    );
  });
  saving.value = calls.length;
  const resps = await Promise.all(calls);
  saving.value = 0;
  if (!resps.every((r) => ok(r.req))) {
    const errs = resps.map((r) => [r.req, r.req.status, r.req.request]);
    toast(`API Error: ${errs}`);
  }
};

const removeSubscription = async (path: AttributePath) => {
  if (ctx.onHead.value) throw new Error("Must be on a change set");

  const apis = createCalls();

  const calls = apis.map(async (call) => {
    const payload = {
      [path]: {
        $source: null,
      },
    };
    return call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  });

  saving.value = calls.length;
  const resps = await Promise.all(calls);
  saving.value = 0;
  if (!resps.every((r) => ok(r.req))) {
    const errs = resps.map((r) => [r.req, r.req.status, r.req.request]);
    toast(`API Error: ${errs}`);
  }
};

const remove = async (path: AttributePath) => {
  const apis = createCalls();
  const payload: componentTypes.UpdateComponentAttributesArgs = {};
  const calls = apis.map(async (call) => {
    payload[path] = { $source: null };
    return call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  });

  saving.value = calls.length;
  const resps = await Promise.all(calls);
  saving.value = 0;
  if (!resps.every((r) => ok(r.req))) {
    const errs = resps.map((r) => [r.req, r.req.status, r.req.request]);
    toast(`API Error: ${errs}`);
  }
};

const onEscape = () => {
  emit("close");
};
onMounted(() => {
  keyEmitter.on("Escape", onEscape);
});
onBeforeUnmount(() => {
  keyEmitter.on("Escape", onEscape);
});

const emit = defineEmits<{
  (e: "close"): void;
}>();
</script>
