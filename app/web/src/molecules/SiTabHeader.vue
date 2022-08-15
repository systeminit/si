<template>
  <Tab v-slot="{ selected }" class="focus:outline-none">
    <span
      :class="classes + ' ' + (selected ? selectedClasses : defaultClasses)"
    >
      <slot />
    </span>
  </Tab>
  <div
    v-if="afterMargin > 0"
    class="border-b border-neutral-300 dark:border-neutral-600"
    :class="'w-' + afterMargin"
  ></div>
</template>

<script setup lang="ts">
import { Tab } from "@headlessui/vue";
import { inject } from "vue";

withDefaults(
  defineProps<{
    classes?: string;
    defaultClasses?: string;
    selectedClasses?: string;
  }>(),
  {
    classes:
      "border-x border-t border-x-neutral-300 border-t-neutral-300 dark:border-x-neutral-600 dark:border-t-neutral-600 h-11 px-2 text-sm inline-flex items-center rounded-t",
    defaultClasses:
      "text-gray-400 border-b border-neutral-300 dark:border-neutral-600",
    selectedClasses: "border-b-white dark:border-b-neutral-800 border-b",
  },
);

const afterMargin = inject("afterMargin", 0);
</script>

<style>
/* TODO (Wendy) - Tailwind classes which seem to not be in our build? BUG! */
.w-2 {
  width: 0.5rem;
}
.w-1 {
  width: 0.25rem;
}
</style>
