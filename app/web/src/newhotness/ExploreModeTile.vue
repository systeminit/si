<template>
  <div
    :class="
      clsx(
        'flex flex-row gap-2xs items-center rounded-sm px-xs py-2xs h-[28px]',
        'cursor-pointer select-none border border-transparent',
        themeClasses('hover:border-action-500', 'hover:border-action-300'),
        selected ? themeClasses('bg-action-200', 'bg-action-900') : themeClasses('bg-neutral-200', 'bg-neutral-800'),
      )
    "
    @click.stop="selected ? null : emit('toggle')"
  >
    <Icon :name="icon" size="sm" />
    <div class="text-sm">{{ label }}</div>
  </div>
</template>

<script setup lang="ts">
import { Icon, IconNames, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType } from "vue";

defineProps({
  icon: { type: String as PropType<IconNames>, required: true },
  label: { type: String, required: true },
  selected: { type: Boolean },
});

const emit = defineEmits<{
  (e: "toggle"): void;
}>();
</script>
