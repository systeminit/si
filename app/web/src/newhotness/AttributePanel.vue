<template>
  <div v-if="root">
    <div class="py-xs">
      <!-- TODO(Wendy) - this doesn't work on the secrets tree yet -->
      <SiSearch
        ref="searchRef"
        v-model="q"
        placeholder="filter attributes..."
        :tabIndex="0"
        @keydown.tab="onTab"
      />
    </div>
    <template
      v-if="'children' in filtered.tree && filtered.tree.children.length > 0"
    >
      <h3
        :class="
          clsx(
            'p-xs border-l',
            themeClasses('bg-neutral-200', 'bg-neutral-800'),
          )
        "
      >
        domain
      </h3>
      <!-- this is _really_ a type guard for "i am not an empty object" -->
      <ul class="border-l">
        <ComponentAttribute
          v-for="child in filtered.tree.children"
          :key="child.id"
          :component="component"
          :attributeTree="child"
          @save="save"
          @delete="remove"
        />
      </ul>
    </template>
    <template v-if="secrets && secrets.children.length > 0">
      <h3
        :class="
          clsx(
            'p-xs border-l',
            themeClasses('bg-neutral-200', 'bg-neutral-800'),
          )
        "
      >
        secrets
      </h3>
      <ul v-if="'children' in secrets" class="border-l">
        <ComponentSecretAttribute
          v-for="secret in secrets.children"
          :key="secret.id"
          :component="component"
          :attributeTree="secret"
        />
      </ul>
    </template>
  </div>
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
import { useRoute } from "vue-router";
import { SiSearch, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import {
  AttributeTree,
  BifrostComponent,
  Prop,
  AttributeValue,
  Secret,
} from "@/workers/types/entity_kind_types";
import { PropKind } from "@/api/sdf/dal/prop";
import { useApi, routes, componentTypes } from "./api_composables";
import ComponentAttribute from "./layout_components/ComponentAttribute.vue";
import { keyEmitter } from "./logic_composables/emitters";
import ComponentSecretAttribute from "./layout_components/ComponentSecretAttribute.vue";

const q = ref("");

const props = defineProps<{
  component: BifrostComponent;
}>();

export interface AttrTree {
  id: string;
  children: AttrTree[];
  parent?: string;
  prop?: Prop;
  secret?: Secret;
  attributeValue: AttributeValue;
  isBuildable: boolean; // is my parent an array or map?
}

const makeAvTree = (
  data: AttributeTree,
  avId: string,
  isBuildable: boolean,
  parent?: string,
): AttrTree => {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const av = data.attributeValues[avId]!;
  const prop = av.propId ? data.props[av.propId] : undefined;
  const secret = av.secret ?? undefined;
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const childrenIds = data.treeInfo[avId]!.children;
  const children = childrenIds.map((id) =>
    makeAvTree(data, id, ["array", "map"].includes(prop?.kind ?? ""), avId),
  );
  const tree: AttrTree = {
    id: avId,
    children,
    parent,
    attributeValue: av,
    prop,
    secret,
    isBuildable,
  };
  return tree;
};

const root = computed<AttrTree>(() => {
  const empty = {
    id: "",
    children: [] as AttrTree[],
    attributeValue: {} as AttributeValue,
    isBuildable: false,
  };
  const raw = props.component.attributeTree;
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

const domain = computed(() =>
  root.value?.children.find((c) => c.prop?.name === "domain"),
);

const secrets = computed(() =>
  root.value?.children.find((c) => c.prop?.name === "secrets"),
);

const filtered = reactive<{ tree: AttrTree | object }>({
  tree: {},
});

watch(
  () => [q.value, domain.value],
  () => {
    if (!q.value) {
      filtered.tree = domain.value ?? {};
      return;
    }
    if (!domain.value) {
      filtered.tree = {};
      return;
    }

    // we need to access attrs by id
    const map: Record<string, AttrTree> = {};
    map[domain.value.id] = domain.value;
    const walking = [...domain.value.children];
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

          if (p.parent && p.id !== domain.value?.id)
            // dont traverse past domain
            parents.push(p.parent);
        }
        prevPid = pId;
      }
    });

    // all roads lead back to domain
    const newDomain = matchesAsTree[domain.value.id];
    filtered.tree = newDomain ?? {};
  },
  { immediate: true },
);

const api = useApi();

const save = async (
  path: string,
  _id: string,
  value: string,
  propKind: PropKind,
  connectingComponentId?: string,
) => {
  const call = api.endpoint<{ success: boolean }>(
    routes.UpdateComponentAttributes,
    { id: props.component.id },
  );

  // TODO - Paul there's a better way to handle this for sure!
  let coercedVal: string | boolean | number = value;
  if (propKind === PropKind.Boolean) {
    coercedVal = value.toLowerCase() === "true" || value === "1";
  } else if (propKind === PropKind.Integer) {
    coercedVal = Math.trunc(Number(value));
  } else if (propKind === PropKind.Float) {
    coercedVal = Number(value);
  }

  const payload: componentTypes.UpdateComponentAttributesArgs = {};
  path = path.replace("root", ""); // endpoint doesn't want it
  payload[path] = coercedVal;
  if (connectingComponentId) {
    payload[path] = {
      $source: { component: connectingComponentId, path: coercedVal },
    };
  }
  await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
};

const route = useRoute();
const remove = async (path: string, _id: string) => {
  const call = api.endpoint<{ success: boolean }>(
    routes.UpdateComponentAttributes,
    { id: props.component.id },
  );
  const payload: componentTypes.UpdateComponentAttributesArgs = {};
  path = path.replace("root", ""); // endpoint doesn't want it
  payload[path] = { $source: null };
  const { req, newChangeSetId } =
    await call.put<componentTypes.UpdateComponentAttributesArgs>(payload);
  if (newChangeSetId && api.ok(req)) {
    api.navigateToNewChangeSet(
      {
        name: "new-hotness-component",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
          componentId: props.component.id,
        },
      },
      newChangeSetId,
    );
  }
};

const searchRef = ref<InstanceType<typeof SiSearch>>();

onMounted(() => {
  keyEmitter.on("Tab", (e) => {
    e.preventDefault();
    searchRef.value?.focusSearch();
  });
});

onBeforeUnmount(() => {
  keyEmitter.off("Tab");
});

const onTab = (e: KeyboardEvent) => {
  // This allows the user to Shift+Tab backwards to the last attribute from the fuzzy search bar
  if (e.shiftKey) {
    e.preventDefault();
    const focusable = Array.from(
      document.querySelectorAll('[tabindex="0"]'),
    ) as HTMLElement[];
    if (focusable) {
      focusable[focusable.length - 1]?.focus();
    }
  }
};
</script>
