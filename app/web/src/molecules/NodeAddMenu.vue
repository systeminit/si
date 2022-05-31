<template>
  <SiMenu :disabled="props.disabled" :tree="generateTree" @selected="onSelect">
    <SiButton
      icon="plus"
      :label="props.addTo ? `Add to ${props.addTo}` : `Add`"
      class="w-full focus:outline-none add-margin-top items-center self-center text-base subpixel-antialiased font-light tracking-tight"
      :class="{
        'text-gray-200': !isOpen,
        'menu-selected': isOpen,
        'text-gray-600': props.disabled ?? false,
        'opacity-50': props.disabled ?? false,
        'cursor-not-allowed': props.disabled ?? false,
      }"
      size="xs"
      data-cy="editor-schematic-node-add-button"
      @click="addMenuClick"
    />
  </SiMenu>
</template>

<script setup lang="ts">
import { MenuFilter, MenuItem, LinkNodeItem } from "@/api/sdf/dal/menu";
import { onBeforeUnmount, ref, computed } from "vue";
import { refFrom, fromRef } from "vuse-rx";
import _ from "lodash";
import { SchematicService } from "@/service/schematic";
import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";
import { combineLatest, from, switchMap } from "rxjs";
import SiButton from "@/atoms/SiButton.vue";
import { editButtonPulse$ } from "@/observable/change_set";
import SiMenu from "@/atoms/SiMenu.vue";
import { SiMenuTree } from "@/utils/menu";

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());

const addMenuClick = () => {
  if (!editMode.value) {
    editButtonPulse$.next(true);
  } else if (!props.disabled) {
    isOpen.value = !isOpen.value;
  }
};

const props = defineProps<{
  disabled?: boolean;
  filter: MenuFilter;
  addTo?: string;
}>();
const emits = defineEmits(["selected"]);

const isOpen = ref<boolean>(false);

const onSelect = (
  obj: { schemaId: number; links?: LinkNodeItem[] },
  event: MouseEvent,
) => {
  event.preventDefault();
  emits("selected", obj.schemaId, event);
  isOpen.value = false;
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

const recursiveParseMenuItems = (parent: SiMenuTree, menuItems: MenuItem[]) => {
  for (const menuItem of menuItems) {
    if (menuItem.kind === "category") {
      const newParent: SiMenuTree = {
        name: menuItem.name,
        kind: "tree",
        children: [],
      };
      recursiveParseMenuItems(newParent, menuItem.items);
      parent.children.push(newParent);
    } else if (menuItem.kind === "item") {
      parent.children.push({
        name: menuItem.name,
        kind: "leaf",
        value: { schemaId: menuItem.schema_id, links: menuItem.links },
      });
    } else if (menuItem.kind === "link") {
      // TODO: find out what this serves for
    }
  }
};
const generateTree = computed(() => {
  const tree: SiMenuTree = { name: "root", kind: "tree", children: [] };
  recursiveParseMenuItems(tree, menuItems.value ?? []);
  return tree;
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
.cursor-not-allowed {
  cursor: not-allowed;
}
</style>
