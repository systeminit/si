<template>
  <div v-if="root">
    <input
      v-model="q"
      class="block w-full border-neutral-300 border-2 my-sm p-xs"
      placeholder="filter attributes..."
    />
    <h3 class="bg-neutral-800 p-xs border-l-2">domain</h3>
    <!-- this is _really_ a type guard for "i am not an empty object" -->
    <ul v-if="'children' in filtered.tree" class="border-l-2">
      <ComponentAttribute
        v-for="child in filtered.tree.children"
        :key="child.id"
        :attributeTree="child"
        @save="save"
      />
    </ul>
    <div v-else>Oh no, no attributes!</div>
  </div>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";
import { computed, reactive, ref, watch } from "vue";
import { Fzf } from "fzf";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  BifrostAttributeTree,
  BifrostComponent,
} from "@/workers/types/dbinterface";
import {
  useApi,
  routes,
  UpdateComponentAttributesArgs,
} from "./api_composables";
import ComponentAttribute from "./layout_components/ComponentAttribute.vue";

const q = ref("");

const props = defineProps<{
  attributeValueId: string;
  component: BifrostComponent;
}>();

const attributeValueId = computed(() => props.attributeValueId);

const attributeTreeMakeKey = useMakeKey();
const attributeTreeMakeArgs = useMakeArgs();
const attributeTreeQuery = useQuery<BifrostAttributeTree | null>({
  queryKey: attributeTreeMakeKey("AttributeTree", attributeValueId),
  queryFn: async () => {
    const args = attributeTreeMakeArgs("AttributeTree", attributeValueId.value);
    return await bifrost<BifrostAttributeTree>(args);
  },
});

const root = computed(() => attributeTreeQuery.data.value);

const domain = computed(() =>
  root.value?.children.find((c) => c.prop?.name === "domain"),
);

const filtered = reactive<{ tree: BifrostAttributeTree | object }>({
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
    const map: Record<string, BifrostAttributeTree> = {};
    map[domain.value.id] = domain.value;
    const walking = [...domain.value.children];
    // walk all the children and find if they match
    while (walking.length > 0) {
      const attr = walking.shift();
      if (!attr) break;
      map[attr.id] = attr;
      walking.push(...attr.children);

      // TODO fuzzy this
      // if (attr.prop?.name.toLowerCase().startsWith(q.value.toLowerCase()))
      // matches.push(attr);
    }

    const fzf = new Fzf(Object.values(map), {
      casing: "case-insensitive",
      selector: (p) =>
        `${p.id} ${p.prop?.name} ${p.prop?.path} ${p.attributeValue.key} ${p.attributeValue.value}`,
    });

    const results = fzf.find(q.value);
    // Maybe we want to get rid of low scoring options (via std dev)?
    const matches: BifrostAttributeTree[] = results.map((fz) => fz.item);

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
      }, {} as Record<string, BifrostAttributeTree>);

    const matchesAsTree: Record<string, BifrostAttributeTree> = {};
    // work backwards from the leaf node, filling in their parents children arrays
    // make sure there are no dupes b/c matches will give us dupes
    matches.forEach((attr) => {
      const parents = [attr.parent];
      let prevPid: string | undefined;
      while (parents.length > 0) {
        const pId = parents.shift();
        if (!pId) throw new Error("no pid");
        let p: BifrostAttributeTree | undefined;
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
);

const api = useApi();

const save = async (path: string, _id: string, value: string) => {
  const call = api.endpoint<{ success: boolean }>(
    routes.UpdateComponentAttributes,
    { id: props.component.id },
  );
  const payload: UpdateComponentAttributesArgs = {};
  path = path.replace("root", ""); // endpoint doesn't want it
  payload[path] = value;
  await call.put<UpdateComponentAttributesArgs>(payload);
};

// attributeValue is not "value" for maps.. look at children! i suspect the same for arrays, etc
// figure out "how to display the details of an array item" rather than just empty
</script>
