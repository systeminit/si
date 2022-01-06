<template>
  <ul v-show="isOpen" :class="listClasses">
    <template v-for="item in menuItems">
      <li
        v-if="item.kind === 'category'"
        :key="item.name"
        class="w-full px-4 text-sm subpixel-antialiased font-light tracking-tight text-left text-gray-300 cursor-pointer options menu-category whitespace-nowrap"
        @mouseenter="open(item.name)"
        @mouseleave="close(item.name)"
      >
        <div class="whitespace-no-wrap hover:text-white">
          {{ item.name }}
        </div>
        <NodeAddMenuCategory
          :menu-items="item.items"
          :is-open="isCategoryOpen(item.name)"
          @selected="selectedType"
        />
      </li>
      <li
        v-if="item.kind === 'item'"
        :key="item.name"
        class="w-full px-4 text-sm subpixel-antialiased font-light tracking-tight text-left text-gray-300 whitespace-no-wrap cursor-pointer options"
        @click="selectedType(item.entityType, $event)"
      >
        <div class="whitespace-no-wrap">
          {{ item.name }}
        </div>
      </li>
    </template>
  </ul>
</template>

<script setup lang="ts">
import { computed, PropType, ref } from "vue";
import { MenuItem } from "@/api/sdf/dal/schematic";

interface OpenCategories {
  [category: string]: boolean;
}

const props = defineProps({
  menuItems: {
    type: Array as PropType<MenuItem[]>,
    required: false,
    default: undefined,
  },
  rootMenu: { type: Boolean, default: false },
  isOpen: { type: Boolean, default: false },
});

const emits = defineEmits(["selected"]);

const openCategories = ref<OpenCategories>({});

const listClasses = computed(() => {
  if (props.rootMenu) {
    return {
      absolute: true,
      "w-auto": true,
      "text-gray-200": true,
      border: true,
      "shadow-md": true,
      options: true,
    };
  } else {
    return {
      relative: true,
      "w-auto": true,
      border: true,
      "shadow-md": true,
      "category-items": true,
      options: true,
    };
  }
});

const selectedType = (entityType: string, event: MouseEvent) => {
  emits("selected", entityType, event);
};
const open = (category: string) => {
  openCategories.value[category] = true;
};
const close = (category: string) => {
  openCategories.value[category] = false;
};
const isCategoryOpen = (category: string) => {
  return openCategories.value[category];
};
</script>
