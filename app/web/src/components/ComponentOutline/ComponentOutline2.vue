<template>
  <div ref="outlineRef" class="flex flex-col">
    <ScrollArea>
      <template #top>
        <SidebarSubpanelTitle class="border-t-0">
          <div class="flex flex-row grow">
            <span class="mr-auto">Diagram Outline</span>
            <Icon
              v-if="fetchComponentsReq.isPending || fixesAreRunning"
              name="loader"
              size="sm"
            />
          </div>
        </SidebarSubpanelTitle>

        <!-- search bar - dont need to show if no components -->
        <SiSearch
          v-if="rootComponents.length"
          autoSearch
          @search="onSearchUpdated"
        />
      </template>

      <template v-if="fetchComponentsReq.isError">
        <ErrorMessage :requestStatus="fetchComponentsReq" />
      </template>
      <template v-else>
        <!-- filtered / search mode -->
        <template v-if="filterModeActive">
          <ComponentOutlineNode
            v-for="component in filteredComponents"
            :key="component.id"
            :componentId="component.id"
          />
        </template>

        <!-- tree mode -->
        <template v-else>
          <div v-if="!rootComponents.length" class="flex flex-col items-center">
            <div class="w-52">
              <EmptyStateIcon name="no-components" />
            </div>
            <div class="text-xl text-neutral-400 dark:text-neutral-300 mt-2">
              Drag & Drop
            </div>
            <div class="text-sm px-xs pt-3 text-neutral-400 text-center italic">
              Drag & Drop assets on to the canvas and start modeling your
              infrastructure
            </div>
            <div class="text-sm px-xs pt-3 text-neutral-400 text-center italic">
              Assets are reusable infrastructure components such as key pairs,
              docker images EC2 instances etc.
            </div>
          </div>
          <template v-else>
            <ComponentOutlineNode
              v-for="component in rootComponents"
              :key="component.id"
              :componentId="component.id"
            />
          </template>
        </template>
      </template>
    </ScrollArea>
  </div>
</template>

<script lang="ts">
type ComponentOutlineRootCtx = {
  filterModeActive: ComputedRef<boolean>;
  itemClickHandler: (e: MouseEvent, id: ComponentId) => void;
};

export const ComponentOutlineCtxInjectionKey: InjectionKey<ComponentOutlineRootCtx> =
  Symbol("ComponentOutlineContext");

export function useComponentOutlineContext2() {
  const ctx = inject(ComponentOutlineCtxInjectionKey, null);
  if (!ctx)
    throw new Error(
      "<ComponentOutlineNode> should only be used within a <ComponentOutline>",
    );
  return ctx;
}
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import {
  computed,
  ComputedRef,
  inject,
  InjectionKey,
  onBeforeUnmount,
  onMounted,
  provide,
  ref,
} from "vue";
import * as _ from "lodash-es";
import { ErrorMessage, Icon, ScrollArea } from "@si/vue-lib/design-system";
import SiSearch from "@/components/SiSearch.vue";
import { ComponentId, useComponentsStore } from "@/store/components.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";

import ComponentOutlineNode from "./ComponentOutlineNode2.vue";
import EmptyStateIcon from "../EmptyStateIcon.vue";

defineProps<{ fixesAreRunning: boolean }>();

const outlineRef = ref<HTMLElement>();

const emit = defineEmits<{
  // while we've avoided events for most things (selection, panning, etc)
  // we still have an emit for this one because the parent (WorkspaceModelAndView) owns the right click menu
  // and needs the raw MouseEvent
  (e: "right-click-item", ev: MouseEvent): void;
}>();

const componentsStore = useComponentsStore();

const fetchComponentsReq =
  componentsStore.getRequestStatus("FETCH_DIAGRAM_DATA");

const rootComponents = computed(() => {
  return _.filter(componentsStore.allComponents, (c) => !c.parentNodeId);
});

const filterString = ref("");
const filterStringCleaned = computed(() =>
  filterString.value.trim().toLowerCase(),
);
const filterModeActive = computed(() => !!filterStringCleaned.value);

const filteredComponents = computed(() => {
  if (!filterModeActive.value) return [];
  return _.filter(componentsStore.allComponents, (c) => {
    if (c.displayName.toLowerCase().includes(filterStringCleaned.value))
      return true;
    if (c.schemaName.toLowerCase().includes(filterStringCleaned.value))
      return true;
    return false;
  });
});

function onSearchUpdated(newFilterString: string) {
  filterString.value = newFilterString;
}

function itemClickHandler(e: MouseEvent, id: ComponentId) {
  // right click
  if (e.button === 2) {
    if (!componentsStore.selectedComponentIds.includes(id)) {
      componentsStore.setSelectedComponentId(id);
    }
    e.preventDefault();
    emit("right-click-item", e);
  } else if (e.shiftKey || e.metaKey) {
    e.preventDefault();
    // TODO: probably want shift-click behaviour to be different
    // ie selecting all items in between 2 clicked items... but can do later
    componentsStore.setSelectedComponentId(id, true); // true = toggle mode
  } else if (e.type === "dblclick") {
    // TODO: probably refactor this to call a fn on an event bus, but this is working for now
    componentsStore.panTargetComponentId = id;
  } else {
    componentsStore.setSelectedComponentId(id);
  }
}

// this object gets provided to all child ComponentOutlineNode instances
// so we dont have to deal with propogating stuff through the tree
const rootCtx = {
  filterModeActive,
  itemClickHandler,
};
provide(ComponentOutlineCtxInjectionKey, rootCtx);

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});

const onKeyDown = (e: KeyboardEvent) => {
  if (document?.activeElement?.tagName !== "BODY") return;

  // Tab goes forwards, Shift-Tab goes backwards
  if (e.key === "Tab") {
    const selectedComponentId = _.last(componentsStore.selectedComponentIds);
    if (!selectedComponentId) return;
    e.preventDefault();

    const componentOutlineNodes = outlineRef.value?.querySelectorAll(
      ".component-outline-node",
    );
    const componentIds = _.map(componentOutlineNodes, (node) =>
      node.getAttribute("data-component-id"),
    );
    const selectedIndex = componentIds.indexOf(selectedComponentId);

    let toSelectIndex = selectedIndex + (e.shiftKey ? -1 : 1);

    if (toSelectIndex < 0) {
      toSelectIndex = componentIds.length - 1;
    } else if (toSelectIndex === componentIds.length) {
      toSelectIndex = 0;
    }
    const toSelect = componentIds[toSelectIndex];
    if (toSelect) {
      componentsStore.setSelectedComponentId(toSelect);
    }
  }
};
</script>
