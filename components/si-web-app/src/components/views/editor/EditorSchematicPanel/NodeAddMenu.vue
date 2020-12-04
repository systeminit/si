<template>
  <div class="relative w-auto">
    <button @click="isOpen = !isOpen" class="w-full focus:outline-none">
      <div
        class="items-center w-full h-5 text-sm subpixel-antialiased font-light tracking-tight text-gray-200"
      >
      Add
      </div>
    </button>

    <div
      v-if="isOpen"
      class="absolute left-0 w-auto border shadow-md -mt-05 options"
    >
      <div
        class="w-full px-4 text-sm subpixel-antialiased font-light tracking-tight text-left text-gray-300 cursor-pointer options hover:text-white"
        v-for="item in menuList"
        :key="item.name"
        @click="onSelect(item)"
      >
        {{ item.id }}
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import _ from "lodash";

import { EntityObject, UiMenuCategory, UiMenuSubCategory } from "si-registry/lib/systemComponent";

type MenuElement = 
  | MenuCategory
  | MenuItem
  | undefined;

interface MenuCategory {
  kind: "category";
  id: string;
  childs: MenuCategory[] |  MenuItem[] | undefined;
}

interface MenuItem {
  kind: "item";
  id: string;
  iEntity: EntityObject;
  parent: MenuCategory | undefined;
}

interface Data {
  isOpen: boolean;
  selected: EntityObject | undefined;
  menuList: MenuElement[];
}

export default Vue.extend({
  name: "NodeAddMenu",
  props: {
    entityTypeList: {
      type: Array as () => EntityObject[],
    },
  },
  components: {
  },
  data(): Data {
    return {
      isOpen: false,
      selected: undefined,
      menuList: [],
    };
  },
  methods: {
    onSelect(entity: EntityObject): void {
      this.selected = entity;
      // this.$emit("selected", option);
      this.isOpen = false;
    },
    menuItemList(): MenuElement[] {
      var filteredList = _.remove(this.entityTypeList, function(entity: EntityObject) {
        if (entity.iEntity?.uiVisible) {
          return entity;
        }
      });
      let sortedList = _.sortBy(filteredList, ["iEntity.uiMenuDisplayName"]);
  
      let menuList: MenuElement[] = [];

      filteredList.forEach(function (entity: EntityObject) {
        if (entity.iEntity?.uiMenuCategory) {

          let menuElement: any = _.find(menuList, function(e: MenuElement) { return e?.id === entity.iEntity?.uiMenuCategory; })
          
          // if (menuElement instanceOf MenuCategory) {
            console.log(menuElement?.kind);
            // console.log(menuElement instanceOf MenuCategory);
          // }
          if (menuElement?.kind === "category" && entity.iEntity.uiMenuDisplayName) {
            let menuItem: MenuItem = { 
              kind: "item",
              id: entity.iEntity.uiMenuDisplayName,
              iEntity: entity,
              parent: menuElement,
            }
            menuElement?.childs?.push(menuItem)

          } else {
            let menuCategory: MenuCategory = { 
              kind: "category",
              id: entity.iEntity.uiMenuCategory,
              childs: [],
            }
            menuList.push(menuCategory)
          }
        }
      });
      return menuList
    },
  },
  mounted() {
    this.menuList = this.menuItemList()
  },
  computed: {},

  // created() {
  //   const handleEscape = e => {
  //     if (e.key === "Esc" || e.key === "Escape") {
  //       this.isOpen = false;
  //     }
  //   };
  //   document.addEventListener("keydown", handleEscape);
  //   this.$once("hook:beforeDestroy", () => {
  //     document.removeEventListener("keydown", handleEscape);
  //   });
  // },
});

</script>

<style>
.menu {
  background-color: #2d3748;
  border-color: #485359;
}

.options {
  background-color: #1f2631;
  border-color: #485359;
}
.options:hover {
  background-color: #3d4b62;
  border-color: #454d3e;
}
</style>
