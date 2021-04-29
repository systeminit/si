<template>
  <div class="relative w-auto z-80">
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

    <ul
      v-show="isOpen"
      class="absolute w-auto ml-2 text-gray-200 border shadow-md options"
    >
      <li
        class="w-full px-4 text-sm subpixel-antialiased font-light tracking-tight text-left text-gray-300 cursor-pointer options menu-category"
        v-for="category in menuItems"
        :key="category.name"
      >
        <div class="whitespace-no-wrap hover:text-white">
          {{ category.name }}
        </div>

        <ul
          v-if="category.items"
          class="relative w-auto border shadow-md category-items options"
        >
          <li
            class="w-full px-4 text-sm subpixel-antialiased font-light tracking-tight text-left text-gray-300 whitespace-no-wrap cursor-pointer options"
            v-for="item in category.items"
            :key="item.entityType"
            @click="onSelect(item, $event)"
          >
            {{ item.displayName }}
          </li>
        </ul>
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import _ from "lodash";

import { entityMenu, MenuCategoryItem } from "si-registry";
import { SchematicKind } from "@/api/sdf/model/schematic";

interface Data {
  isOpen: boolean;
}

export interface AddMenuSelectedPayload {
  entityType: string;
  event: MouseEvent;
}

export enum MenuFilter {
  Deployment = "deployment",
  Implementation = "implementation",
}

export default Vue.extend({
  name: "NodeAddMenu",
  props: {
    disabled: {
      type: Boolean,
      default: false,
    },
    filter: {
      type: Array as PropType<SchematicKind[]>,
    },
  },
  components: {},
  data(): Data {
    return {
      isOpen: false,
    };
  },
  computed: {
    menuItems(): ReturnType<typeof entityMenu>["list"] {
      const result = entityMenu(this.filter);
      return result.list;
    },
  },
  methods: {
    onSelect({ entityType }: { entityType: string }, event: MouseEvent): void {
      event.preventDefault();

      const payload: AddMenuSelectedPayload = {
        entityType: entityType,
        event: event,
      };
      this.$emit("selected", entityType, event);
      this.isOpen = false;
    },
    onMouseEnter(): void {
      this.isOpen = true;
    },
  },
  created() {
    const _handleEscape = (e: any) => {
      if (e.key === "Esc" || e.key === "Escape") {
        if (this.isOpen) {
          this.isOpen = false;
        }
      }
    };
    // document.addEventListener("keydown", handleEscape);
    // this.$once("hook:beforeDestroy", () => {
    //   document.removeEventListener("keydown", handleEscape);
    // });
  },
});
</script>

<style>
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
  visibility: hidden;
  position: absolute;
  left: 100%;
  top: auto;
  margin-top: -1.305rem;
}

.menu-category:hover .category-items {
  visibility: visible;
}
</style>
