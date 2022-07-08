<template>
  <MenuButton class="flex text-sm rounded-full border-black border-2">
    <span class="sr-only">Open user menu</span>
    <img
      class="h-8 w-8 rounded-full bg-white"
      :src="CheechSvg"
      alt="Cheech and Chong"
    />
  </MenuButton>
</template>

<script setup lang="ts">
import { MenuButton } from "@headlessui/vue";
import CheechSvg from "@/assets/images/cheech-and-chong.svg";
</script>
