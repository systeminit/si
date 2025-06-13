<template>
  <ul class="p-xs">
    <CodeViewer
      v-if="resourcePayload"
      :code="JSON.stringify(resourcePayload, null, 2)"
    />
    <EmptyState v-else icon="check-hex" text="This component has no resource" />
  </ul>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import {
  AttributeTree,
  BifrostComponent,
} from "@/workers/types/entity_kind_types";
import CodeViewer from "@/components/CodeViewer.vue";
import { findAvsAtPropPath } from "./util";
import EmptyState from "./EmptyState.vue";

const props = defineProps<{
  component: BifrostComponent;
  attributeTree?: AttributeTree;
}>();

const resourcePayload = computed(() => {
  if (!props.attributeTree) return;
  const data = findAvsAtPropPath(props.attributeTree, [
    "root",
    "resource",
    "payload",
  ]);
  if (!data) return;
  const { attributeValues } = data;
  // only one AV for /resource/payload
  return attributeValues[0]?.value;
});
</script>
