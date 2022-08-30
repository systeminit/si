<template>
  <CodeViewer :code="logs.join('\n')">
    <template #title>
      <div
        v-if="status === 'success'"
        class="flex flex-row whitespace-nowrap items-center"
      >
        <div class="pr-2 text-success-500"><Icon name="check-circle" /></div>
        Success!
      </div>
      <div
        v-else-if="status === 'failure'"
        class="flex flex-row whitespace-nowrap items-center"
      >
        <div class="pr-2 text-destructive-500"><Icon name="x-circle" /></div>
        Failure
      </div>
      <div v-else class="flex flex-row whitespace-nowrap items-center">
        <div class="pr-2"><Icon name="loader" /></div>
        Running...
      </div>
    </template>
  </CodeViewer>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import CodeViewer from "@/organisms/CodeViewer.vue";
import Icon from "@/ui-lib/Icon.vue";

export type WorkflowStatus = "running" | "success" | "failure";

defineProps({
    logs: { type: Array<string>, required: true },
    status: {type: String as PropType<"running" | "success" | "failure">, required: true}
});
</script>
