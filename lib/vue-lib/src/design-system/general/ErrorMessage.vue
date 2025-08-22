<template>
  <div
    v-if="computedMessage || $slots.default"
    :class="
      clsx(
        'flex flex-row items-center',
        variant === 'classic' && [
          'text-sm border rounded-sm',
          !noPadding && 'p-xs',
          tone === 'destructive' &&
            'border-destructive-500 text-destructive-400',
          tone === 'warning' && 'border-warning-500 text-warning-400',
          tone === 'info' && 'border-action-900 text-action-200',
        ],
        variant === 'block' && [
          'p-2xs text-xs',
          tone === 'warning' &&
            themeClasses(
              'bg-warning-300 text-warning-800',
              'bg-warning-900 text-warning-200',
            ),
          tone === 'destructive' &&
            'bg-destructive-300 text-destructive-900 dark:bg-destructive-900 dark:text-destructive-200',
          tone === 'success' && 'text-success-200 bg-success-900',
          tone === 'info' && 'bg-action-900 text-action-200',
        ],
      )
    "
  >
    <Icon v-if="!noIcon" :name="props.icon" class="mr-xs flex-none" />
    <div class="flex-grow">
      <slot>{{ computedMessage }}</slot>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { UseAsyncStateReturn } from "@vueuse/core";
import { computed } from "vue";
import clsx from "clsx";
import { ApiRequestStatus, getErrorMessage } from "../../pinia";
import { Tones } from "../utils/color_utils";
import { Icon, themeClasses, IconNames } from "..";

export type ErrorMessageVariant = "classic" | "block";

const props = withDefaults(
  defineProps<{
    message?: string;
    requestStatus?: ApiRequestStatus;
    asyncState?: UseAsyncStateReturn<unknown, unknown[], boolean>;
    tone?: Tones;
    noPadding?: boolean;
    variant?: ErrorMessageVariant;
    icon?: IconNames;
    noIcon?: boolean;
  }>(),
  {
    tone: "destructive",
    variant: "classic",
    icon: "alert-triangle",
  },
);

const computedMessage = computed(() => props.message ?? getErrorMessage(props));
</script>
