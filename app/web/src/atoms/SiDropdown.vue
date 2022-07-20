<template>
  <MenuItems
    class="z-10 absolute rounded-lg shadow-lg py-1ring-1 ring-black ring-opacity-5 focus:outline-none mt-1 bg-black min-w-full"
  >
    <MenuItem
      v-for="option in props.options"
      v-slot="{ active }"
      :key="option.text"
    >
      <a
        :class="[
          active ? 'bg-[#2F80ED] rounded-md' : '',
          'block whitespace-nowrap px-4 py-2 text-sm text-white cursor-pointer',
        ]"
        @click="option.action"
        >{{ option.text }}</a
      >
    </MenuItem>
  </MenuItems>
</template>

<script setup lang="ts">
import { MenuItem, MenuItems } from "@headlessui/vue";

export interface SiDropdownOption {
  text: string;
  action: (() => Promise<void>) | (() => void);
}

const props = defineProps<{
  options: SiDropdownOption[];
}>();
</script>
