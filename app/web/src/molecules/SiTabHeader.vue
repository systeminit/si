<template>
  <Tab v-slot="{ selected }" class="focus:outline-none">
    <span
      :class="classesF + ' ' + (selected ? selectedClassesF : defaultClassesF)"
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

const props = defineProps<{
  classes?: string;
  defaultClasses?: string;
  selectedClasses?: string;
}>();

const afterMargin = inject("afterMargin", 0);
const classesF = inject("tabClasses", props.classes);
const defaultClassesF = inject("defaultTabClasses", props.defaultClasses);
const selectedClassesF = inject("selectedTabClasses", props.selectedClasses);
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
