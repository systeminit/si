<template>
  <button
    :class="
      clsx(
        'flex flex-row gap-xs items-center border rounded-sm',
        'p-2xs mb-[-1px] h-7',
        'font-mono text-[13px] text-left truncate relative',
        themeClasses('border-neutral-400 hover:border-action-500', 'border-neutral-600 hover:border-action-300'),
        selected ? themeClasses('bg-action-200', 'bg-action-900') : themeClasses('bg-neutral-100', 'bg-neutral-900'),
      )
    "
  >
    <Icon name="repeat" size="xs" />
    <span>See default subscriptions</span>
  </button>
</template>

<script lang="ts" setup>
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";

defineProps<{
  selected?: boolean;
}>();
</script>
