<template>
  <div
    class="h-full flex flex-row p-2 rounded items-center justify-between bg-neutral-800 overflow-hidden"
  >
    <div class="h-full flex flex-row whitespace-nowrap pr-2">
      <WorkflowStatusIcon :status="status" />
      <span>{{ name }}</span>
    </div>
    <div class="whitespace-nowrap text-ellipsis overflow-hidden">
      {{ timestamp ? timestamp : "Not Run Yet" }}
    </div>
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
