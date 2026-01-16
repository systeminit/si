<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <component :is="ResizablePanel" ref="leftResizablePanelRef" rememberSizeKey="func-picker" side="left" :minSize="320">
    <div class="flex flex-col h-full">
      <div class="relative flex-grow">
        <CustomizeTabs tabContentSlug="packages">
          <template #packages>
            <ModuleListPanel />
          </template>
        </CustomizeTabs>
      </div>
    </div>
  </component>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 font-semi-bold flex flex-col relative"
  >
    <ModuleDisplay :key="moduleSlug" />
  </div>
  <component
    :is="ResizablePanel"
    ref="rightResizablePanelRef"
    rememberSizeKey="func-details"
    side="right"
    :minSize="200"
  >
    <SidebarSubpanelTitle label="Module Details" />
    <ModuleDetailsPanel :key="moduleSlug" />
  </component>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { ResizablePanel } from "@si/vue-lib/design-system";
import ModuleListPanel from "@/components/modules/ModuleListPanel.vue";
import ModuleDisplay from "@/components/modules/ModuleDisplay.vue";
import ModuleDetailsPanel from "@/components/modules/ModuleDetailsPanel.vue";
import { useModuleStore } from "@/store/module.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import CustomizeTabs from "../CustomizeTabs.vue";

const moduleStore = useModuleStore();
const moduleSlug = computed(() => moduleStore.urlSelectedModuleSlug);

const leftResizablePanelRef = ref();
const rightResizablePanelRef = ref();

const onKeyDown = async (e: KeyboardEvent) => {
  if (e.altKey && e.shiftKey && leftResizablePanelRef.value && rightResizablePanelRef.value) {
    if (leftResizablePanelRef.value.collapsed && rightResizablePanelRef.value.collapsed) {
      // Open all panels
      leftResizablePanelRef.value.collapseSet(false);
      rightResizablePanelRef.value.collapseSet(false);
      leftResizablePanelRef.value.subpanelCollapseSet(false);
    } else {
      // Close all panels
      leftResizablePanelRef.value.collapseSet(true);
      rightResizablePanelRef.value.collapseSet(true);
    }
  }
};

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});
</script>
