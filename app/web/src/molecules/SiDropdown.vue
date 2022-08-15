<template>
  <!-- FIXME(nick,victor): remove reliance on z index -->
  <MenuItems
    class="z-30 absolute rounded-md shadow-lg ring-1 py-1 ring-black ring-opacity-5 focus:outline-none mt-1 bg-black min-w-full"
  >
    <slot />
  </MenuItems>
</template>

<script setup lang="ts">
import { MenuItems } from "@headlessui/vue";
</script>
