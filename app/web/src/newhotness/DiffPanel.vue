<template>
  <ul class="p-xs flex flex-col gap-xs">
    <CodeViewer v-if="diff" :code="diff" codeLanguage="diff" />
    <EmptyStateCard
      v-else
      iconName="no-components"
      primaryText="No diff to display."
      secondaryText="No diff was found for this component"
    />
  </ul>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import CodeViewer from "@/components/CodeViewer.vue";
import EmptyStateCard from "@/components/EmptyStateCard.vue";
import { BifrostComponent } from "@/workers/types/dbinterface";

const component = defineProps<{
  component: BifrostComponent;
}>();

const diff = computed(() => {
  return component.component.resourceDiff.diff;
});
</script>
