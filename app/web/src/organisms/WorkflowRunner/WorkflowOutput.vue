<template>
  <CodeViewer :code="logs.join('\n')">
    <template #title>
      <WorkflowStatusIcon :status="status" show-text />
    </template>
  </CodeViewer>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import CodeViewer from "@/organisms/CodeViewer.vue";
import WorkflowStatusIcon from "@/molecules/WorkflowStatusIcon.vue";

defineProps({
    logs: { type: Array<string>, required: true },
    status: {type: String as PropType<"running" | "success" | "failure">, required: true}
});
</script>
