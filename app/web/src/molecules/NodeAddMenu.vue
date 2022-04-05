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
        class="add-margin-top items-center self-center w-full text-sm subpixel-antialiased font-light tracking-tight"
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
import { onBeforeUnmount, PropType, ref } from "vue";
import { refFrom, fromRef } from "vuse-rx";
import _ from "lodash";
import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
import { combineLatest, from, switchMap } from "rxjs";

const props = defineProps({
  disabled: { type: Boolean, default: false },
  filter: {
    type: Object as PropType<MenuFilter>,
    required: true,
  },
});
const emits = defineEmits(["selected"]);

const isOpen = ref<boolean>(false);

const onSelect = (schemaId: number, event: MouseEvent) => {
  event.preventDefault();
  emits("selected", schemaId, event);
  isOpen.value = false;
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const debounceIsOpen = _.debounce((component: any, isOpen: boolean) => {
  component.value = isOpen;
}, 500);
const onMouseLeave = () => {
  debounceIsOpen(isOpen, false);
};
const cancelClose = () => {
  debounceIsOpen.cancel();
};

const props$ = fromRef(props, { immediate: true, deep: true });

const menuItems = refFrom<MenuItem[] | undefined>(
  combineLatest([props$]).pipe(
    switchMap(([props]) => {
      return SchematicService.getNodeAddMenu({ menuFilter: props.filter });
    }),
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([undefined]);
      } else {
        return from([response]);
      }
    }),
  ),
);

const handleEscape = (e: KeyboardEvent) => {
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
.add-margin-top {
  margin-top: 0.25rem;
}

/* .menu-category:hover .category-items { */
/*   visibility: visible; */
/* } */
</style>
