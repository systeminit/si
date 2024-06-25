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
        ],
      )
    "
  >
    <Icon name="alert-triangle" class="mr-xs flex-none" />
    <div class="flex-grow">
      <slot>{{ computedMessage }}</slot>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import clsx from "clsx";
import { ApiRequestStatus } from "../../pinia";
import { Tones } from "../utils/color_utils";
import { Icon, themeClasses } from "..";

export type ErrorMessageVariant = "classic" | "block";

const props = defineProps({
  message: { type: String },
  requestStatus: { type: Object as PropType<ApiRequestStatus> },
  tone: { type: String as PropType<Tones>, default: "destructive" },
  noPadding: { type: Boolean },
  variant: {
    type: String as PropType<ErrorMessageVariant>,
    default: "classic",
  },
});

const computedMessage = computed(() => {
  if (props.message) return props.message;
  return props.requestStatus?.errorMessage;
});
</script>
