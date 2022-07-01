<template>
  <MenuButton
    class="bg-gray-800 flex text-sm rounded-full focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white"
  >
    <span class="sr-only">Open user menu</span>
    <img class="h-8 w-8 rounded-full bg-white" :src="CheechSvg" alt="" />
  </MenuButton>
</template>

<script setup lang="ts">
import { MenuButton } from "@headlessui/vue";
import CheechSvg from "@/assets/images/cheech-and-chong.svg";
</script>
