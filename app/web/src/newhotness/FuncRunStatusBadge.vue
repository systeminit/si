<template>
  <div
    v-if="computedStatus"
    class="flex items-center rounded-sm px-1.5 py-0.5"
    :class="computedClasses"
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
import { IconSizes, themeClasses } from "@si/vue-lib/design-system";
import { computed } from "vue";
import clsx from "clsx";
import { tw } from "@si/vue-lib";
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

const computedClasses = computed(() => {
  const status = computedStatus.value ?? "";

  return clsx(
    "border",
    (status === "Failure" || status === "ActionFailure") &&
      themeClasses(
        tw`bg-destructive-200 border-destructive-400 text-destructive-700`,
        tw`bg-destructive-900 border-destructive-700 text-destructive-400`,
      ),
    status === "Success" &&
      themeClasses(
        tw`bg-success-200 border-success-400 text-success-700`,
        tw`bg-success-900 border-success-700 text-success-400`,
      ),
    (status === "Running" || status === "Postprocessing") &&
      themeClasses(
        tw`bg-action-200 border-action-400 text-action-700`,
        tw`bg-action-900 border-action-700 text-action-400`,
      ),
    ![
      "Failure",
      "Success",
      "Running",
      "Postprocessing",
      "ActionFailure",
    ].includes(status) &&
      themeClasses(
        tw`bg-neutral-200 border-neutral-400 text-neutral-700`,
        tw`bg-neutral-900 border-neutral-700 text-neutral-400`,
      ),
  );
});
</script>
