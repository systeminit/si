<template>
  <ul class="p-xs flex flex-col gap-xs">
    <template v-if="component.id && codes">
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
import { computed } from "vue";
import {
  AttributeTree,
  BifrostComponent,
} from "@/workers/types/entity_kind_types";
import CodeViewer from "@/components/CodeViewer.vue";
import EmptyStateCard from "@/components/EmptyStateCard.vue";
import { findAvsAtPropPath } from "./util";

const props = defineProps<{
  component: BifrostComponent;
  attributeTree?: AttributeTree;
}>();

const codes = computed(() => {
  const codes: { code: string; name: string | undefined }[] = [];
  if (!props.attributeTree) return codes;

  const data = findAvsAtPropPath(props.attributeTree, [
    "root",
    "code",
    "codeItem",
    "code",
  ]);
  if (!data) return codes;
  const { attributeValues } = data;

  attributeValues.forEach((av) => {
    codes.push({
      code: av.value ?? "",
      name: av.path?.split("/")[2], // "/code/<name>/code"
    });
  });

  return codes;
});
</script>
