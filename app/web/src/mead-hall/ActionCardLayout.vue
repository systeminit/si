<template>
  <div
    :class="
      clsx(
        'flex flex-row items-center text-sm relative p-2xs min-w-0 w-full border border-transparent',
        !noInteraction
          ? 'cursor-pointer hover:border-action-500 dark:hover:border-action-300 group/actioncard'
          : '',
        selected
          ? 'dark:bg-action-900 bg-action-100 border-action-500 dark:border-action-300'
          : 'dark:border-neutral-800',
        actionFailed ? 'action-failed' : '',
      )
    "
  >
    <slot name="icons"> </slot>

    <div class="flex flex-col flex-grow min-w-0">
      <TruncateWithTooltip class="w-full">
        <span class="font-bold"> {{ abbr }}: </span>
        <span
          :class="
            clsx(
              themeClasses('text-neutral-700', 'text-neutral-200'),
              !noInteraction &&
                themeClasses(
                  'group-hover/actioncard:text-action-500',
                  'group-hover/actioncard:text-action-300',
                ),
            )
          "
        >
          <template v-if="componentId">
            {{ componentSchemaName }}
            {{ componentName ?? "unknown" }}
            {{ description }}
          </template>
        </span>
      </TruncateWithTooltip>
      <div v-if="actor" class="text-neutral-500 dark:text-neutral-400 truncate">
        <span class="font-bold">By:</span> {{ actor }}
      </div>
    </div>

    <slot name="interaction"> </slot>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { themeClasses, TruncateWithTooltip } from "@si/vue-lib/design-system";

defineProps<{
  noInteraction?: boolean;
  selected?: boolean;
  actionFailed: boolean;
  componentId: string | null | undefined;
  componentSchemaName?: string;
  componentName?: string;
  description?: string;
  actor?: string;
  abbr: string;
}>();
</script>
