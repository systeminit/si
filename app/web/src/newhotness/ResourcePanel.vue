<template>
  <ul class="p-xs">
    <CodeViewer
      v-if="resourcePayload"
      :code="JSON.stringify(resourcePayload, null, 2)"
    />
    <EmptyStateCard
      v-else
      iconName="no-changes"
      primaryText="No Resource"
      secondaryText="This component does not have a resource associated with it."
    />
  </ul>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { BifrostComponent } from "@/workers/types/entity_kind_types";
import CodeViewer from "@/components/CodeViewer.vue";
import EmptyStateCard from "@/components/EmptyStateCard.vue";
import { findAvsAtPropPath } from "./util";

const props = defineProps<{
  component: BifrostComponent;
}>();

const resourcePayload = computed(() => {
  if (!props.component.attributeTree) return;
  const data = findAvsAtPropPath(props.component.attributeTree, [
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
