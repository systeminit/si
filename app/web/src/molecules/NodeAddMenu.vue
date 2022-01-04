<template>
  <div
    class="relative w-auto menu-root"
    @mouseleave="onMouseLeave"
    @mouseenter="cancelClose"
  >
    <button
      class="w-full focus:outline-none"
      data-cy="editor-schematic-node-add-button"
      :disabled="disabled"
      @click="isOpen = !isOpen"
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
      :is-open="isOpen"
      :menu-items="menuItems"
      root-menu
      class="menu-root"
      @selected="onSelect"
    />
  </div>
</template>

<script setup lang="ts">
import { MenuFilter, MenuItem } from "@/api/sdf/dal/schematic";
import NodeAddMenuCategory from "./NodeAddMenu/NodeAddMenuCategory.vue";
import { computed, onBeforeUnmount, PropType, ref } from "vue";
import { refFrom } from "vuse-rx";
import _ from "lodash";
import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
import { tap } from "rxjs";
import { GetNodeAddMenuResponse } from "@/service/schematic/get_node_add_menu";
import { ApiResponse } from "@/api/sdf";

const props = defineProps({
  disabled: { type: Boolean, default: false },
  filter: {
    type: Object as PropType<MenuFilter>,
    required: true,
  },
});
const emits = defineEmits(["selected"]);

const isOpen = ref<boolean>(false);

const onSelect = (entityType: string, event: MouseEvent) => {
  event.preventDefault();
  emits("selected", entityType, event);
  isOpen.value = false;
};

const debounceIsOpen = _.debounce((component: any, isOpen: boolean) => {
  component.value = isOpen;
}, 500);
const onMouseLeave = () => {
  debounceIsOpen(isOpen, false);
};
const cancelClose = () => {
  debounceIsOpen.cancel();
};

const menuItems = refFrom(
  SchematicService.getNodeAddMenu({ menuFilter: props.filter }).pipe(
    tap((response: ApiResponse<GetNodeAddMenuResponse>) => {
      if (response.error) {
        GlobalErrorService.set(response);
      }
    }),
  ),
);

//    computed<MenuItem[]>(() => {
//  return [
//    {
//      kind: "category",
//      name: "Snoopy",
//      items: [
//        {
//          kind: "item",
//          name: "floopy",
//          entityType: "floopy",
//        },
//      ],
//    },
//  ];
//});

const handleEscape = (e: any) => {
  if (e.key === "Esc" || e.key === "Escape") {
    if (isOpen.value) {
      isOpen.value = false;
    }
  }
};
document.addEventListener("keydown", handleEscape);
onBeforeUnmount(() => {
  document.removeEventListener("keydown", handleEscape);
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
