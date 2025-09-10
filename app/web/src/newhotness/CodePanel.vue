<template>
  <ul class="p-xs flex flex-col gap-xs">
    <template v-if="component.id && codes && codes.length > 0">
      <CodeViewer
        v-for="(item, index) in codes"
        :key="item.name ?? index"
        :title="item.name ?? `${index + 1}`"
        :showTitle="codes.length > 1 || !!item.name"
        :code="item.code?.toString()"
        titleClasses="text-sm font-bold h-8"
      />
    </template>
    <EmptyState v-else icon="code-transform" text="No code was generated yet" />
  </ul>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import {
  AttributeTree,
  BifrostComponent,
  JsonValue,
} from "@/workers/types/entity_kind_types";
import CodeViewer from "@/components/CodeViewer.vue";
import { findAvsAtPropPath } from "./util";
import EmptyState from "./EmptyState.vue";

const props = defineProps<{
  component: BifrostComponent;
  attributeTree?: AttributeTree;
}>();

const codes = computed(() => {
  const codes: { code: JsonValue; name: string | undefined }[] = [];
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
