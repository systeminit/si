<template>
  <ul :class="listClasses" v-show="isOpen">
    <template v-for="item in menuItems">
      <li
        class="w-full px-4 text-sm subpixel-antialiased font-light tracking-tight text-left text-gray-300 cursor-pointer options menu-category whitespace-nowrap"
        v-if="item.kind == 'category'"
        @mouseenter="open(item.name)"
        @mouseleave="close(item.name)"
        :key="item.name"
      >
        <div class="whitespace-no-wrap hover:text-white">
          {{ item.name }}
        </div>
        <NodeAddMenuCategory
          :menuItems="item.items"
          :isOpen="isCategoryOpen(item.name)"
          @selected="selectedType"
        />
      </li>
      <li
        class="w-full px-4 text-sm subpixel-antialiased font-light tracking-tight text-left text-gray-300 whitespace-no-wrap cursor-pointer options"
        v-if="item.kind == 'item'"
        :key="item.name"
        @click="selectedType(item.entityType, $event)"
      >
        <div class="whitespace-no-wrap">
          {{ item.name }}
        </div>
      </li>
    </template>
  </ul>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { MenuItem } from "si-registry";

interface Data {
  openCategories: {
    [category: string]: boolean;
  };
}

export default Vue.extend({
  name: "NodeAddMenuCategory",
  props: {
    menuItems: {
      type: Array as PropType<MenuItem[]>,
    },
    rootMenu: { type: Boolean, default: false },
    isOpen: { type: Boolean },
  },
  data(): Data {
    return {
      openCategories: {},
    };
  },
  computed: {
    listClasses(): Record<string, boolean> {
      if (this.rootMenu) {
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
    },
  },
  methods: {
    selectedType(entityType: string, event: MouseEvent) {
      this.$emit("selected", entityType, event);
    },
    open(category: string): void {
      Vue.set(this.openCategories, category, true);
    },
    close(category: string): void {
      Vue.set(this.openCategories, category, false);
    },
    isCategoryOpen(category: string): boolean {
      return this.openCategories[category];
    },
  },
});
</script>
