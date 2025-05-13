<template>
  <ul class="p-xs flex flex-col gap-xs">
    <template v-if="componentId && codes">
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
import { AttributeTree } from "@/workers/types/dbinterface";
import CodeViewer from "@/components/CodeViewer.vue";
import EmptyStateCard from "@/components/EmptyStateCard.vue";
import { findAvsAtPropPath } from "./util";

const props = defineProps<{
  componentId: string;
}>();

const attributeTreeMakeKey = makeKey("AttributeTree", props.componentId);
const attributeTreeMakeArgs = makeArgs("AttributeTree", props.componentId);
const attributeTreeQuery = useQuery<AttributeTree | null>({
  queryKey: attributeTreeMakeKey,
  queryFn: async () => await bifrost<AttributeTree>(attributeTreeMakeArgs),
});

const root = computed(() => attributeTreeQuery.data.value);

const codes = computed(() => {
  const codes: { code: string; name: string | undefined }[] = [];
  if (!root.value) return codes;

  const data = findAvsAtPropPath(root.value, [
    "root",
    "code",
    "codeItem",
    "code",
  ]);
  if (!data) return codes;
  const { attributeValues } = data;

  attributeValues.forEach((av) => {
    codes.push({
      code: av.value,
      name: av.path?.split("/")[2], // "/code/<name>/code"
    });
  });

  return codes;
});
</script>
