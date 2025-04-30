<template>
  <div
    v-if="computedStatus"
    class="flex items-center rounded-sm px-1.5 py-0.5"
    :class="{
      'bg-destructive-900 border border-destructive-700 text-destructive-400':
        computedStatus === 'Failure' || computedStatus === 'ActionFailure',
      'bg-success-900 border border-success-700 text-success-400':
        computedStatus === 'Success',
      'bg-action-900 border border-action-700 text-action-400':
        computedStatus === 'Running' || computedStatus === 'Postprocessing',
      'bg-neutral-900 border border-neutral-700 text-neutral-400': ![
        'Failure',
        'Success',
        'Running',
        'Postprocessing',
        'ActionFailure',
      ].includes(computedStatus),
    }"
  >
    <StatusIndicatorIcon
      :status="computedStatus"
      type="management"
      :size="size"
      class="mr-1.5"
    />
    <span class="text-xs font-medium">{{ computedStatus }}</span>
  </div>
</template>

<script lang="ts" setup>
import { IconSizes } from "@si/vue-lib/design-system";
import { computed } from "vue";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { FuncRunState } from "@/store/func_runs.store";

const props = withDefaults(
  defineProps<{
    status: string | FuncRunState | null | undefined;
    type?: string;
    size?: IconSizes;
  }>(),
  {
    type: "management",
    size: "xs",
  },
);

// Make sure we have a valid status value
const computedStatus = computed<string | null>(() => {
  if (!props.status) return null;

  // Return the status string if it's valid
  return props.status;
});
</script>
