<template>
  <div
    :class="
      clsx(
        'cursor-pointer w-9 h-6 flex items-center  rounded-full p-1 duration-300 ease-in-out',
        props.selected
          ? 'dark:bg-action-400 bg-action-300'
          : 'dark:bg-neutral-300 bg-neutral-600',
      )
    "
    @click="click"
  >
    <div
      :class="
        clsx(
          'dark:bg-white bg-black w-4 h-4 rounded-full shadow-md transform duration-300 ease-in-out',
          props.selected ? 'translate-x-3' : '',
        )
      "
    ></div>
  </div>
</template>

<script setup lang="ts">
import clsx from "clsx";

type clickFn = () => void;

const props = defineProps<{
  selected: boolean;
  click: clickFn;
}>();
</script>
