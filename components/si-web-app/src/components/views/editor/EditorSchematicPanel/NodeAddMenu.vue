<template>
  <div class="relative w-auto">
    <button
      @click="isOpen = !isOpen"
      class="w-full focus:outline-none"
      :disabled="disabled"
    >
      <div
        class="items-center mx-2 w-full self-center text-sm subpixel-antialiased font-light tracking-tight"
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
      v-if="isOpen"
      class="absolute ml-2 w-auto border shadow-md options text-gray-200"
      @mouseleave="onMouseLeave"
    >
      <li
        class="w-full px-4 text-sm subpixel-antialiased font-light tracking-tight text-left text-gray-300 cursor-pointer options menu-category"
        v-for="item in menuList"
        :key="item.id"
      >
        <div class="hover:text-white whitespace-no-wrap">
          {{ item.id }}
        </div>

        <ul
          v-if="item.childs"
          class="relative category-items border shadow-md options w-auto"
        >
          <li
            class="w-full px-4 text-sm subpixel-antialiased font-light tracking-tight text-left text-gray-300 cursor-pointer options whitespace-no-wrap"
            v-for="child in item.childs"
            :key="child.id"
            @click="onSelect(child)"
          >
            {{ child.id }}
          </li>
        </ul>
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import _ from "lodash";

import {
  EntityObject,
  UiMenuCategory,
  UiMenuSubCategory,
} from "si-registry/lib/systemComponent";

type MenuElement = MenuCategory | MenuItem | undefined;

interface MenuCategory {
  kind: "category";
  id: string;
  childs: MenuCategory[] | MenuItem[] | undefined;
}

interface MenuItem {
  kind: "item";
  id: string;
  iEntity: EntityObject;
  parent: MenuCategory | undefined;
}

interface Data {
  isOpen: boolean;
  selected: MenuItem | undefined;
  menuList: MenuElement[];
}

export default Vue.extend({
  name: "NodeAddMenu",
  props: {
    entityTypeList: {
      type: Array as () => EntityObject[],
    },
    disabled: {
      type: Boolean,
      default: false,
    },
  },
  components: {},
  data(): Data {
    return {
      isOpen: false,
      selected: undefined,
      menuList: [],
    };
  },
  methods: {
    onSelect(item: MenuItem): void {
      this.$emit("selected", item.iEntity);
      this.isOpen = false;
    },
    menuItemList(): MenuElement[] {
      // Filter the entityTypeList for entities that are uiVisible
      var filteredList = _.remove(this.entityTypeList, function(
        entity: EntityObject,
      ) {
        if (entity.iEntity?.uiVisible) {
          return entity;
        }
      });

      // Sort the filtered list
      let sortedList = _.sortBy(filteredList, ["iEntity.uiMenuDisplayName"]);

      let menuList: MenuElement[] = [];

      sortedList.forEach(function(entity: EntityObject) {
        // Check if this entity has a uiMenuCategory
        if (entity.iEntity?.uiMenuCategory) {
          let menuElement: any;
          // Find the menuElement tht represents the entity uiMenuCategory.
          menuElement = _.find(menuList, function(e: MenuElement) {
            return e?.id === entity.iEntity?.uiMenuCategory;
          });

          // Create a menuElement to represent the entity uiMenuCategory if it doesn't already exist
          if (!menuElement) {
            if (
              entity.iEntity?.uiMenuCategory &&
              menuElement?.kind !== "category"
            ) {
              let menuCategory: MenuCategory = {
                kind: "category",
                id: entity.iEntity.uiMenuCategory,
                childs: [],
              };
              menuList.push(menuCategory);
              menuList = _.sortBy(menuList, ["id"]);
              menuElement = _.find(menuList, function(e: MenuElement) {
                return e?.id === entity.iEntity?.uiMenuCategory;
              });
            }
          }

          // Add menuItem to MenuCategory
          if (
            menuElement?.kind === "category" &&
            entity.iEntity.uiMenuDisplayName
          ) {
            let menuItemId: string;

            if (entity.iEntity.uiMenuSubCategory) {
              menuItemId =
                entity.iEntity.uiMenuSubCategory +
                " : " +
                entity.iEntity.uiMenuDisplayName;
            } else {
              menuItemId = entity.iEntity.uiMenuDisplayName;
            }

            let menuItem: MenuItem = {
              kind: "item",
              id: menuItemId,
              iEntity: entity,
              parent: menuElement,
            };
            menuElement?.childs?.push(menuItem);
          }
        }
      });
      return menuList;
    },
    onMouseLeave(): void {
      this.isOpen = false;
    },
  },
  mounted() {
    this.menuList = this.menuItemList();
  },
  created() {
    const handleEscape = (e: any) => {
      if (e.key === "Esc" || e.key === "Escape") {
        if(this.isOpen) {
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
  z-index: 1000;
}

.menu-category:hover .category-items {
  visibility: visible;
}
</style>
