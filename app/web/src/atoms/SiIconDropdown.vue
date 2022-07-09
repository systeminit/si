<template>
  <MenuItems class="z-10" :class="menuItemsClass">
    <div v-for="option in props.options" :key="option.text">
      <MenuItem v-slot="{ active }">
        <a
          v-if="option.action"
          :class="[
            active ? 'bg-gray-100' : '',
            'block px-4 py-2 text-sm text-gray-700',
          ]"
          @click="option.action"
          >{{ option.text }}</a
        >

        <a
          v-else
          :class="[
            active ? 'bg-gray-100' : '',
            'block px-4 py-2 text-sm text-gray-700',
          ]"
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
  menuItemsClass?: string;
}>();

const defaultClass =
  "origin-top-right absolute right-0 mt-2 w-48 rounded-md shadow-lg py-1 bg-white ring-1 ring-black ring-opacity-5 focus:outline-none";

const menuItemsClass = computed((): string => {
  if (props.menuItemsClass) {
    return props.menuItemsClass;
  }
  return defaultClass;
});
</script>
