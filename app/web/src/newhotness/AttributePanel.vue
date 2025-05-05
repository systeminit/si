<template>
  <div v-if="root">
    <input
      v-model="q"
      class="block w-full border-neutral-300 border-2 my-sm p-xs"
      placeholder="filter attributes..."
    />
    <h3 class="bg-neutral-800 p-xs border-l-2">domain</h3>
    <ul v-if="filtered" class="border-l-2">
      <ComponentAttribute
        v-for="child in filtered.children"
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
import { computed, ref } from "vue";
import { bifrost, makeArgs, makeKey } from "@/store/realtime/heimdall";
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

const attributeTreeMakeKey = makeKey("AttributeTree", props.attributeValueId);
const attributeTreeMakeArgs = makeArgs("AttributeTree", props.attributeValueId);
const attributeTreeQuery = useQuery<BifrostAttributeTree | null>({
  queryKey: attributeTreeMakeKey,
  queryFn: async () =>
    await bifrost<BifrostAttributeTree>(attributeTreeMakeArgs),
});

const root = computed(() => attributeTreeQuery.data.value);

const domain = computed(() =>
  root.value?.children.find((c) => c.prop?.name === "domain"),
);

const filtered = computed<BifrostAttributeTree | null>(() => {
  if (!q.value) return domain.value ?? null;
  if (!domain.value) return null;

  const matches: BifrostAttributeTree[] = [];

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
    if (attr.prop?.name.toLowerCase().startsWith(q.value.toLowerCase()))
      matches.push(attr);
  }

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
  return newDomain ?? null;
});

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
