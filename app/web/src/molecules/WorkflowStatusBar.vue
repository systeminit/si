<template>
  <div
    ref="statusBar"
    class="h-full flex flex-row p-2 rounded items-center justify-between bg-neutral-800 overflow-hidden"
  >
    <div class="h-full flex flex-row whitespace-nowrap pr-2">
      <WorkflowStatusIcon :status="status" />
      <span>{{ name }}</span>
    </div>
    <div
      ref="timestampDiv"
      class="whitespace-nowrap text-ellipsis overflow-hidden"
    >
      <Timestamp v-if="timestamp" :date="new Date(timestamp)" size="extended" />
      <span v-else>Not Run Yet</span>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import _ from "lodash";
import WorkflowStatusIcon from "@/molecules/WorkflowStatusIcon.vue";
import Timestamp from "@/ui-lib/Timestamp.vue";

defineProps({
  name: { type: String, required: true },
  status: {
    type: String as PropType<"running" | "success" | "failure">,
    required: true,
  },
  timestamp: { type: String },
});
</script>
