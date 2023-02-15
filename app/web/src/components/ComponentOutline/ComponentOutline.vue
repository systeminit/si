<template>
  <div class="flex flex-col">
    <template v-if="fetchComponentsReq.isPending && !rootComponents.length"
      >Loading...</template
    >
    <template v-else-if="fetchComponentsReq.isError">
      <ErrorMessage :request-status="fetchComponentsReq" />
    </template>
    <template v-else>
      <!-- search bar - dont need to show if no components -->
      <template v-if="rootComponents.length">
        <SiSearch auto-search @search="onSearchUpdated" />
      </template>

      <!-- filtered / search mode -->
      <template v-if="filterModeActive">
        <ComponentOutlineNode
          v-for="component in filteredComponents"
          :key="component.id"
          :component-id="component.id"
        />
      </template>

      <!-- tree mode -->
      <div v-else class="">
        <template v-if="!rootComponents.length">
          <div class="p-2 text-neutral-500">Your model is currently empty</div>
        </template>
        <template v-else>
          <ComponentOutlineNode
            v-for="component in rootComponents"
            :key="component.id"
            :component-id="component.id"
          />
        </template>
      </div>
    </template>
  </div>
</template>

<script lang="ts">
type ComponentOutlineRootCtx = {
  filterModeActive: ComputedRef<boolean>;
  itemClickHandler: (e: MouseEvent, id: ComponentId) => void;
};

export const ComponentOutlineCtxInjectionKey: InjectionKey<ComponentOutlineRootCtx> =
  Symbol("ComponentOutlineContext");

export function useComponentOutlineContext() {
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
import { computed, ComputedRef, inject, InjectionKey, provide, ref } from "vue";
import _ from "lodash";
import SiSearch from "@/components/SiSearch.vue";

import { ComponentId, useComponentsStore } from "@/store/components.store";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import ComponentOutlineNode from "./ComponentOutlineNode.vue";

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
</script>
