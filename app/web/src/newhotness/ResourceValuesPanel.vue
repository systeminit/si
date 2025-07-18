<template>
  <div
    v-if="root && component && attributeTree && component.hasResource"
    class="p-xs flex flex-col gap-xs"
  >
    <div>
      <SiSearch
        ref="searchRef"
        v-model="q"
        placeholder="filter resource values..."
        :tabIndex="0"
        :borderBottom="false"
        @keydown.tab="onSearchInputTab"
      />
    </div>
    <div
      v-if="'children' in filtered.tree && filtered.tree.children.length > 0"
    >
      <!-- this is _really_ a type guard for "i am not an empty object" -->
      <ul
        :class="
          clsx(
            'border-l border-r border-t',
            themeClasses('border-neutral-300', 'border-neutral-800'),
          )
        "
      >
        <ComponentAttribute
          v-for="child in filtered.tree.children"
          :key="child.id"
          :component="component"
          :attributeTree="child"
          forceReadOnly
        />
      </ul>
      <div
        :class="
          clsx(
            'w-full border-b',
            themeClasses('border-neutral-300', 'border-neutral-800'),
          )
        "
      />
    </div>
  </div>
  <EmptyState
    v-else
    icon="check-hex"
    text="No resource"
    secondaryText="This component hasn’t been applied to HEAD, so its resource (the real-world object it represents) hasn’t been created yet."
  />
</template>

<script lang="ts" setup>
import {
  computed,
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
  watch,
} from "vue";
import { Fzf } from "fzf";
import { themeClasses, SiSearch } from "@si/vue-lib/design-system";
import clsx from "clsx";
import * as _ from "lodash-es";
import {
  AttributeTree,
  AttributeValue,
  BifrostComponent,
} from "@/workers/types/entity_kind_types";
import ComponentAttribute from "./layout_components/ComponentAttribute.vue";
import { keyEmitter } from "./logic_composables/emitters";
import { AttrTree, makeAvTree } from "./logic_composables/attribute_tree";
import EmptyState from "./EmptyState.vue";

const q = ref("");

const props = defineProps<{
  component?: BifrostComponent;
  attributeTree?: AttributeTree;
}>();

// TODO(nick): move the root computation to a shared location since this is a copy from "AttributePanel".
const root = computed<AttrTree>(() => {
  const empty = {
    id: "",
    componentId: "",
    children: [] as AttrTree[],
    attributeValue: {} as AttributeValue,
    isBuildable: false,
  };
  const raw = props.attributeTree;
  if (!raw) return empty;

  // find the root node in the tree, the only one with parent null
  const rootId = Object.keys(raw.treeInfo).find((avId) => {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    const av = raw.treeInfo[avId]!;
    if (!av.parent) return true;
    return false;
  });
  if (!rootId) return empty;

  const tree = makeAvTree(raw, rootId, false);
  return tree;
});

const resourceValue = computed(() =>
  root.value?.children.find((c) => c.prop?.name === "resource_value"),
);

const filtered = reactive<{ tree: AttrTree | object }>({
  tree: {},
});

// TODO(nick): move the root computation to a shared location since this is a copy from "AttributePanel".
watch(
  () => [q.value, resourceValue.value],
  () => {
    if (!q.value) {
      filtered.tree = resourceValue.value ?? {};
      return;
    }
    if (!resourceValue.value) {
      filtered.tree = {};
      return;
    }

    // we need to access attrs by id
    const map: Record<string, AttrTree> = {};
    map[resourceValue.value.id] = resourceValue.value;
    const walking = [...resourceValue.value.children];
    // walk all the children and find if they match
    while (walking.length > 0) {
      const attr = walking.shift();
      if (!attr) break;
      map[attr.id] = attr;
      walking.push(...attr.children);
    }

    const fzf = new Fzf(Object.values(map), {
      casing: "case-insensitive",
      selector: (p) =>
        `${p.id} ${p.prop?.name} ${p.prop?.path} ${p.attributeValue.key} ${p.attributeValue.value}`,
    });

    const results = fzf.find(q.value);
    // Maybe we want to get rid of low scoring options (via std dev)?
    const matches: AttrTree[] = results.map((fz) => fz.item);

    // get new instances of all the objects with empty children arrays
    const parentsWithoutChildren = Object.values(map)
      .map((attr) => {
        return {
          ...attr,
          children: [],
        };
      })
      .reduce((map, attr) => {
        map[attr.id] = attr;
        return map;
      }, {} as Record<string, AttrTree>);

    const matchesAsTree: Record<string, AttrTree> = {};
    // work backwards from the leaf node, filling in their parents children arrays
    // make sure there are no dupes b/c matches will give us dupes
    matches.forEach((attr) => {
      const parents = [attr.parent];
      let prevPid: string | undefined;
      while (parents.length > 0) {
        const pId = parents.shift();
        if (!pId) throw new Error("no pid");
        let p: AttrTree | undefined;
        p = matchesAsTree[pId];
        if (!p) p = parentsWithoutChildren[pId];
        if (p) {
          if (prevPid) {
            const lastParent = matchesAsTree[prevPid];
            if (lastParent && !p.children.some((c) => c.id === lastParent.id))
              p.children.push(lastParent);
          } else if (!p.children.some((c) => c.id === attr.id))
            p.children.push(attr);

          matchesAsTree[p.id] = p;

          if (p.parent && p.id !== resourceValue.value?.id)
            // dont traverse past "resource_value"
            parents.push(p.parent);
        }
        prevPid = pId;
      }
    });

    // all roads lead back to resource value
    const newResourceValue = matchesAsTree[resourceValue.value.id];
    filtered.tree = newResourceValue ?? {};
  },
  { immediate: true },
);

const searchRef = ref<InstanceType<typeof SiSearch>>();

onMounted(() => {
  keyEmitter.on("Tab", (e) => {
    e.preventDefault();
    focusSearch();
  });
  focusSearch();
});

const focusSearch = () => {
  searchRef.value?.focusSearch();
};

onBeforeUnmount(() => {
  keyEmitter.off("Tab");
});

const onSearchInputTab = (e: KeyboardEvent) => {
  if (e.shiftKey) {
    e.preventDefault();
    focusSearch();
  }
};
</script>
