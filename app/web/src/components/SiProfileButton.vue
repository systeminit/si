<template>
  <MenuButton
    v-slot="{ open }"
    class="flex flex-row items-center text-sm h-full pl-2 pr-1 hover:bg-black"
    @mouseenter="toggleHover"
    @mouseleave="toggleHover"
  >
    <span class="sr-only">Open user menu</span>
    <CheechSvg class="h-8 w-8 rounded-full bg-shade-0 border-shade-100 border-2" />

    <SiArrow :nudge="hovered === true || open" class="ml-1 w-4 text-white" />
  </MenuButton>
</template>

<script lang="ts" setup>
import { MenuButton } from "@headlessui/vue";
import { ref } from "vue";
import CheechSvg from "@/assets/images/cheech-and-chong.svg?component";
import SiArrow from "./SiArrow.vue";

const hovered = ref<boolean>(false);
const toggleHover = () => {
  hovered.value = !hovered.value;
};
</script>
