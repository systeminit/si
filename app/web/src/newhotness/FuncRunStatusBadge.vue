<template>
  <div
    :class="
      clsx(
        'flex flex-row gap-2xs items-center border rounded-sm px-2xs py-3xs',
        themeClasses(
          'text-neutral-800 bg-neutral-50',
          'text-neutral-100 bg-neutral-900',
        ),
        status === 'Success' &&
          themeClasses('border-success-500', 'border-success-800'),
        status === 'Failure' &&
          themeClasses('border-destructive-500', 'border-destructive-600'),
        (status === 'Running' || status === 'Unknown') && 'border-neutral-600',
      )
    "
  >
    <Icon
      size="2xs"
      :name="iconName"
      :class="
        clsx(
          status === 'Success' && 'text-success-400',
          status === 'Failure' && 'text-destructive-400',
        )
      "
    />
    <span class="text-xs">{{ status }}</span>
  </div>
</template>

<script lang="ts" setup>
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import { computed } from "vue";
import clsx from "clsx";
import { FuncRunState } from "@/store/func_runs.store";

const props = defineProps<{
  status: string | FuncRunState | null | undefined;
}>();

type Status = "Success" | "Failure" | "Running" | "Unknown";

const status = computed<Status>(() => {
  if (props.status === "Success") return "Success";
  if (props.status === "Failure" || props.status === "ActionFailure")
    return "Failure";
  if (props.status === "Running" || props.status === "Postprocessing")
    return "Running";
  return "Unknown";
});

const iconName = computed(() => {
  if (status.value === "Success") return "circle-full";
  if (status.value === "Failure") return "triangle";
  if (status.value === "Running") return "loader";
  return "question-circle";
});
</script>
