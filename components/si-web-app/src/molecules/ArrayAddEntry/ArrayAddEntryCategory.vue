<template>
  <ul :class="listClasses" v-show="isOpen && menuItems">
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
        <ArrayAddEntryCategory
          :menuItems="item.items"
          :isOpen="isCategoryOpen(item.name)"
          @selected="selectedType"
        />
      </li>
      <li
        class="w-full px-4 text-sm subpixel-antialiased font-light tracking-tight text-left text-gray-300 whitespace-no-wrap cursor-pointer options"
        v-if="item.kind == 'item'"
        :key="item.name"
        @click="selectedType(item.name, $event)"
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
import { EditPartial } from "si-registry/dist/registryEntry";

interface Data {
  openCategories: {
    [category: string]: boolean;
  };
}

export default Vue.extend({
  name: "ArrayAddEntryCategory",
  props: {
    menuItems: {
      type: Array as PropType<EditPartial[]>,
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
          inline: true,
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
    selectedType(name: string, event: MouseEvent) {
      this.$emit("selected", name, event);
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
