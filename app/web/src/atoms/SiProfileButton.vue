<template>
  <MenuButton
    v-slot="{ open }"
    class="flex flex-row items-center text-sm h-full pl-2 pr-1 hover:bg-black"
    @mouseenter="toggleHover"
    @mouseleave="toggleHover"
  >
    <span class="sr-only">Open user menu</span>
    <img
      class="h-8 w-8 rounded-full bg-white border-black border-2"
      :src="CheechSvg"
      alt="Cheech and Chong"
    />

    <SiArrow :nudge="hovered === true || open" class="ml-1 w-4 text-white" />
  </MenuButton>
</template>

<script setup lang="ts">
import { MenuButton } from "@headlessui/vue";
import CheechSvg from "@/assets/images/cheech-and-chong.svg";
import { ref } from "vue";
import SiArrow from "@/atoms/SiArrow.vue";

const hovered = ref<boolean>(false);
const toggleHover = () => {
  hovered.value = !hovered.value;
};
</script>
