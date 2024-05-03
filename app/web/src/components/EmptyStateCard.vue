<template>
  <div class="flex flex-col items-center m-sm gap-xs">
    <div class="w-64 flex flex-col items-center">
      <EmptyStateIcon :name="iconName" />
    </div>
    <div
      class="text-lg font-medium text-center text-neutral-600 dark:text-neutral-300"
    >
      {{ primaryText }}
    </div>
    <div
      class="text-sm italic text-neutral-500 dark:text-neutral-400 text-center"
    >
      {{ secondaryText }}
    </div>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import EmptyStateIcon, { EMPTY_STATE_ICON_NAMES } from "./EmptyStateIcon.vue";

defineProps({
  iconName: {
    type: String as PropType<EMPTY_STATE_ICON_NAMES>,
    required: true,
  },
  primaryText: { type: String, required: true },
  secondaryText: { type: String, required: true },
});
</script>
