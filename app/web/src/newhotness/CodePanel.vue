<template>
  <ul class="p-xs flex flex-col gap-xs">
    <template v-if="attributeValueId && codes">
      <CodeViewer
        v-for="(item, index) in codes"
        :key="item.name ?? index"
        :title="item.name ?? `${index + 1}`"
        :showTitle="codes.length > 1 || !!item.name"
        :code="item.code"
        titleClasses="text-sm font-bold h-8"
      />
    </template>
    <EmptyStateCard
      v-else
      iconName="no-changes"
      primaryText="No Code"
      secondaryText="This component has not generated any code yet."
    />
  </ul>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";
import { bifrost, makeArgs, makeKey } from "@/store/realtime/heimdall";
import { BifrostAttributeTree } from "@/workers/types/dbinterface";
import CodeViewer from "@/components/CodeViewer.vue";
import EmptyStateCard from "@/components/EmptyStateCard.vue";

const props = defineProps<{
  attributeValueId?: string;
}>();

const attributeTreeMakeKey = makeKey("AttributeTree", props.attributeValueId);
const attributeTreeMakeArgs = makeArgs("AttributeTree", props.attributeValueId);
const attributeTreeQuery = useQuery<BifrostAttributeTree | null>({
  queryKey: attributeTreeMakeKey,
  queryFn: async () =>
    await bifrost<BifrostAttributeTree>(attributeTreeMakeArgs),
});

const root = computed(() => attributeTreeQuery.data.value);

const codes = computed(() => {
  const codeValueTree = root.value?.children.find(
    (child) => child.prop?.name === "code",
  );
  const itemTrees = codeValueTree?.children.filter(
    (child) => child.prop?.name === "codeItem",
  );
  const codes: { code: string; name: string | undefined }[] = [];
  itemTrees?.forEach((tree) => {
    const code = tree?.children.find((child) => child.prop?.name === "code")
      ?.attributeValue.value;
    if (code)
      codes.push({
        code,
        name: tree.attributeValue.key,
      });
  });
  if (codes.length === 0) return undefined;
  return codes;
});
</script>
