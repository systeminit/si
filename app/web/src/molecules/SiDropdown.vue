<template>
  <!-- FIXME(nick,victor): remove reliance on z index -->
  <MenuItems
    class="z-30 absolute rounded-md shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none mt-1 min-w-full overflow-clip"
    :class="classes"
  >
    <slot />
  </MenuItems>
</template>

<script setup lang="ts">
import { MenuItems } from "@headlessui/vue";
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    navbar?: boolean;
  }>(),
  {
    navbar: true,
  },
);

const classes = computed(() =>
  props.navbar
    ? "bg-black"
    : "bg-neutral-50 dark:bg-neutral-900 text-black dark:text-white border border-neutral-600",
);
</script>
