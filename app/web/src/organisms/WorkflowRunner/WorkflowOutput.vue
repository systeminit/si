<template>
  <CodeViewer :code="logs.join('\n')" :force-theme="forceTheme">
    <template #title>
      <WorkflowStatusIcon :status="status" show-text />
    </template>
  </CodeViewer>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import CodeViewer from "@/organisms/CodeViewer.vue";
import WorkflowStatusIcon from "@/molecules/WorkflowStatusIcon.vue";
import { ThemeValue } from "@/observable/theme";

defineProps({
    logs: { type: Array<string>, required: true },
    status: {type: String as PropType<"running" | "success" | "failure">, required: true},
    forceTheme: { type: String as PropType<ThemeValue> },
});
</script>
