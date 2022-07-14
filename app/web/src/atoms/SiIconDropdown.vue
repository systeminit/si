<template>
  <MenuItems
    :class="dropdownClasses"
    class="z-10 absolute rounded-b-md shadow-lg py-1ring-1 ring-black ring-opacity-5 focus:outline-none"
  >
    <div v-for="option in props.options" :key="option.text">
      <MenuItem v-slot="{ active }">
        <a
          :class="[
            active ? 'bg-gray-700 rounded-md' : '',
            'block px-4 py-2 text-sm text-white',
          ]"
          @click="option.action"
          >{{ option.text }}</a
        >
      </MenuItem>
    </div>
  </MenuItems>
</template>

<script setup lang="ts">
import { MenuItem, MenuItems } from "@headlessui/vue";
import { defineProps } from "vue";
import { SiIconDropdownOption } from "@/atoms/SiIconDropdown/types";
import { computed } from "@vue/reactivity";

const props = defineProps<{
  options: SiIconDropdownOption[];
  profileMode?: boolean;
}>();

const dropdownClasses = computed((): string => {
  if (props.profileMode) {
    return "origin-top-right right-0 top-[3rem] bg-[#333333] w-48";
  }
  return "top-[5.75rem] bg-black min-w-full";
});
</script>
