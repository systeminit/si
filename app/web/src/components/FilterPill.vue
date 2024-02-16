<template>
  <div
    :class="
      clsx(
        'flex flex-row items-center rounded-full border w-fit pr-2xs pl-[2px] py-[1px] text-xs cursor-pointer',
        selected
          ? 'bg-action-500 dark:bg-action-400 border-action-500 dark:border-action-400'
          : 'hover:text-action-400 hover:dark:text-action-300 hover:border-action-400 hover:dark:border-action-300 border-action-700 dark:border-action-200',
      )
    "
  >
    <Icon v-if="iconName" :name="iconName" size="sm" :class="iconClasses" />
    <div class="pr-2xs">
      <slot name="label">{{ label }}</slot>
    </div>
    <div v-if="number || number === 0" class="font-bold">{{ number }}</div>
  </div>
</template>

<script lang="ts" setup>
import {
  Icon,
  IconNames,
  Tones,
  getToneTextColorClass,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType, computed } from "vue";

const props = defineProps({
  label: { type: String },
  number: { type: Number },
  iconTone: { type: String as PropType<Tones> },
  iconName: { type: String as PropType<IconNames> },
  selected: { type: Boolean },
});

const iconClasses = computed(() => {
  if (props.iconTone && !props.selected) {
    return getToneTextColorClass(props.iconTone);
  } else return "";
});
</script>
