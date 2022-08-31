<template>
  <div
    class="h-full flex flex-row p-2 border border-neutral-600 rounded items-center justify-between"
  >
    <div class="h-full flex flex-row">
      <WorkflowStatusIcon :status="status" />
      <span>{{ name }}</span>
    </div>
    <div>{{ timestamp ? timestamp : "Not Run Yet" }}</div>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import WorkflowStatusIcon from "@/molecules/WorkflowStatusIcon.vue";

defineProps({
  name: { type: String, required: true },
  status: {
    type: String as PropType<"running" | "success" | "failure">,
    required: true,
  },
  timestamp: { type: String },
});
</script>
