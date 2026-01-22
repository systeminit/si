<template>
  <div :class="clsx('flex flex-row items-center')">
    <div
      :class="
        clsx(
          size === 'sm' ? 'w-6 h-4' : 'w-9 h-6',
          'cursor-pointer flex-none flex items-center  rounded-full p-1 duration-300 ease-in-out',
          props.selected
            ? themeClasses('bg-action-300', 'bg-action-500')
            : themeClasses('bg-neutral-400', 'bg-neutral-500'),
        )
      "
      @click="click"
    >
      <div
        :class="
          clsx(
            'bg-white rounded-full shadow-md transform duration-300 ease-in-out',
            size === 'sm'
              ? [
                  'w-3 h-3',
                  props.selected
                    ? 'translate-x-[0.35rem]'
                    : 'translate-x-[-0.1rem]',
                ]
              : ['w-4 h-4', props.selected ? 'translate-x-3' : ''],
          )
        "
      ></div>
    </div>
    <span
      :class="clsx('px-1 text-xs', props.labelWidth)"
      v-if="props.onLabel && props.selected"
      >{{ props.onLabel }}</span
    >
    <span
      :class="clsx('px-1 text-xs', props.labelWidth)"
      v-if="props.offLabel && !props.selected"
      >{{ props.offLabel }}</span
    >
  </div>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { PropType } from "vue";
import { themeClasses } from "../utils/theme_tools";

type clickFn = (e: MouseEvent) => void;

const props = defineProps({
  size: { type: String, default: "md" },
  selected: { type: Boolean },
  click: { type: Function as PropType<clickFn>, default: () => {} },
  onLabel: { type: String, required: false },
  offLabel: { type: String, required: false },
  labelWidth: { type: String, required: false, default: "" },
});
</script>
