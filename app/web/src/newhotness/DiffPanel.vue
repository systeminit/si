<template>
  <ul class="p-xs flex flex-col gap-xs">
    <CodeViewer v-if="diff" :code="diff" codeLanguage="diff" />
    <EmptyState v-else icon="diff" text="No diff information yet" />
  </ul>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import CodeViewer from "@/components/CodeViewer.vue";
import { BifrostComponent } from "@/workers/types/entity_kind_types";
import EmptyState from "./EmptyState.vue";

const component = defineProps<{
  component: BifrostComponent;
}>();

const diff = computed(() => {
  return component.component.resourceDiff.diff;
});
</script>
