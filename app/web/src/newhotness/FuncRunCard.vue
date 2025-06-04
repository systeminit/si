<template>
  <div
    class=""
    :class="
      clsx(
        'group/funcruncard relative p-xs border border-transparent cursor-pointer transition-all duration-200',
        themeClasses(
          'hover:border-action-500 border-b-neutral-300',
          'hover:border-action-300 border-b-neutral-800',
        ),
        isRunning
          ? 'border-l-4 border-l-action-500 pl-xs animate-[pulse_2s_infinite]'
          : 'border-l-1',
      )
    "
    @click="$emit('click', funcRun.id)"
  >
    <div class="flex items-center gap-xs">
      <!-- Status indicator -->
      <div>
        <StatusIndicatorIcon
          :status="funcRunStatus(funcRun)"
          type="management"
          size="sm"
        />
      </div>

      <!-- Function info -->
      <div class="flex-grow">
        <div class="flex items-center justify-between">
          <div
            :class="
              clsx(
                'text-sm font-medium',
                themeClasses(
                  'group-hover/funcruncard:text-action-500',
                  'group-hover/funcruncard:text-action-300',
                ),
              )
            "
            :title="funcRun.functionDisplayName || funcRun.functionName"
          >
            {{ funcRun.functionDisplayName || funcRun.functionName }}
          </div>
          <div class="text-xs text-neutral-500 pl-xs">
            {{ formatTimeAgo(funcRun.createdAt) }}
          </div>
        </div>

        <div class="grow flex items-center text-xs gap-xs">
          <!-- Kind badge -->
          <span
            class="px-2xs py-3xs rounded-full text-2xs inline-flex items-center justify-center"
            :class="functionKindClass(funcRun.functionKind)"
          >
            {{ funcRun.functionKind }}
          </span>

          <span v-if="funcRun.componentId && funcRun.componentName">
            {{ funcRun.componentName }}
          </span>

          <span
            v-if="funcRun.actionId"
            class="flex items-center text-action-400"
          >
            <Icon name="bolt" size="xs" />
            Action
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { computed } from "vue";
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { funcRunStatus, FuncRun } from "./api_composables/func_run";

const props = defineProps<{
  funcRun: FuncRun;
}>();

defineEmits<{
  (e: "click", funcRunId: string): void;
}>();

/**
 * Determines if the function is currently in a running state
 * @returns {boolean} True if function is in a running state
 */
const isRunning = computed(() => {
  return ["Created", "Dispatched", "Running", "Postprocessing"].includes(
    props.funcRun.state,
  );
});

/**
 * Formats a timestamp to a relative time string (e.g., "5m ago")
 * @param {string} dateString - ISO date string to format
 * @returns {string} Formatted relative time
 */
const formatTimeAgo = (dateString: string): string => {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();

  // Convert to seconds
  const diffSec = Math.floor(diffMs / 1000);

  if (diffSec < 60) {
    return `${diffSec}s ago`;
  }

  // Convert to minutes
  const diffMin = Math.floor(diffSec / 60);

  if (diffMin < 60) {
    return `${diffMin}m ago`;
  }

  // Convert to hours
  const diffHour = Math.floor(diffMin / 60);

  if (diffHour < 24) {
    return `${diffHour}h ago`;
  }

  // Convert to days
  const diffDay = Math.floor(diffHour / 24);
  return `${diffDay}d ago`;
};

/**
 * Returns Tailwind CSS classes for a function kind badge
 * @param {string} kind - The function kind
 * @returns {string} Tailwind CSS classes for the badge
 */
const functionKindClass = (kind: string): string => {
  const classes = {
    action: "bg-action-900 text-action-300",
    attribute: "bg-success-900 text-success-300",
    authentication: "bg-warning-900 text-warning-300",
    management: "bg-neutral-800 text-neutral-300",
    intrinsic: "bg-neutral-800 text-neutral-300",
    codeGeneration: "bg-violet-900 text-violet-300",
  };

  return (
    classes[kind as keyof typeof classes] || "bg-neutral-800 text-neutral-300"
  );
};
</script>
