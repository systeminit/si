<template>
  <div
    class="relative w-auto menu-root"
    @mouseleave="onMouseLeave"
    @mouseenter="cancelClose"
  >
    <button
      @click="isOpen = !isOpen"
      class="w-full focus:outline-none"
      :disabled="disabled"
      data-cy="editor-schematic-node-add-button"
    >
      <div
        class="items-center self-center w-full text-sm subpixel-antialiased font-light tracking-tight"
        :class="{
          'text-gray-200': !isOpen,
          'menu-selected': isOpen,
          'text-gray-600': disabled,
        }"
      >
        Add
      </div>
    </button>

    <NodeAddMenuCategory
      :isOpen="isOpen"
      :menuItems="menuItems"
      rootMenu
      class="menu-root"
      @selected="onSelect"
    />
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import _ from "lodash";

import { entityMenu, MenuList } from "si-registry";
import NodeAddMenuCategory from "./NodeAddMenu/NodeAddMenuCategory.vue";

interface Data {
  isOpen: boolean;
}

export interface AddMenuSelectedPayload {
  entityType: string;
  event: MouseEvent;
}

const debounceIsOpen = _.debounce((component: any, isOpen: boolean) => {
  component.isOpen = isOpen;
}, 500);

export default Vue.extend({
  name: "NodeAddMenu",
  props: {
    disabled: {
      type: Boolean,
      default: false,
    },
    filter: {
      type: Object as PropType<Parameters<typeof entityMenu>[0]>,
    },
  },
  components: {
    NodeAddMenuCategory,
  },
  data(): Data {
    return {
      isOpen: false,
    };
  },
  computed: {
    menuItems(): MenuList["list"] {
      return entityMenu(this.filter).list;
    },
  },
  methods: {
    onSelect(entityType: string, event: MouseEvent): void {
      event.preventDefault();
      this.$emit("selected", entityType, event);
      this.isOpen = false;
    },
    onMouseLeave(): void {
      debounceIsOpen(this, false);
    },
    cancelClose(): void {
      debounceIsOpen.cancel();
    },
  },
  created() {
    const handleEscape = (e: any) => {
      if (e.key === "Esc" || e.key === "Escape") {
        if (this.isOpen) {
          this.isOpen = false;
        }
      }
    };
    document.addEventListener("keydown", handleEscape);
    this.$once("hook:beforeDestroy", () => {
      document.removeEventListener("keydown", handleEscape);
    });
  },
});
</script>

<style>
.menu-root {
  z-index: 999;
}

.menu {
  background-color: #2d3748;
  border-color: #485359;
}

.menu-selected {
  background-color: #edf2f8;
  color: #000000;
}

.menu-not-selected {
  color: red;
}

.options {
  background-color: #1f2631;
  border-color: #485359;
}
.options:hover {
  background-color: #3d4b62;
  border-color: #454d3e;
}

.menu-category .category-items {
  /*  visibility: hidden; */
  position: absolute;
  left: 100%;
  top: auto;
  margin-top: -1.305rem;
}

.menu-category:hover .category-items {
  /* visibility: visible; */
}
</style>
