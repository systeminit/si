<template>
  <Tab v-slot="{ selected }" class="focus:outline-none">
    <span
      :class="classes + ' ' + (selected ? selectedClasses : defaultClasses)"
    >
      <slot />
    </span>
  </Tab>
</template>

<script setup lang="ts">
import { Tab } from "@headlessui/vue";

withDefaults(
  defineProps<{
    classes?: string;
    defaultClasses?: string;
    selectedClasses?: string;
  }>(),
  {
    classes: "text-sm inline-flex items-center",
    defaultClasses: "text-gray-400",
    selectedClasses: "font-bold text-black dark:text-white",
  },
);
</script>
