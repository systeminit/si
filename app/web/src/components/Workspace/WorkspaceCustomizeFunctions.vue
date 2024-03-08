<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <component
    :is="ResizablePanel"
    ref="leftResizablePanelRef"
    rememberSizeKey="func-picker"
    side="left"
    :minSize="300"
  >
    <div class="flex flex-col h-full">
      <div class="relative flex-grow">
        <CustomizeTabs tabContentSlug="functions">
          <FuncListPanel />
        </CustomizeTabs>
      </div>
    </div>
  </component>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 font-semi-bold flex flex-col relative"
  >
    <div class="left-2 right-2 top-2 bottom-2 absolute">
      <FuncEditorTabs />
    </div>
  </div>
  <component
    :is="ResizablePanel"
    ref="rightResizablePanelRef"
    rememberSizeKey="func-details"
    side="right"
    :minSize="200"
  >
    <div class="absolute w-full flex flex-col h-full">
      <SidebarSubpanelTitle label="Function Details" />

      <FuncDetails
        :key="funcStore.urlSelectedFuncId"
        :funcId="funcStore.urlSelectedFuncId"
        singleModelScreen
      />
    </div>
  </component>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { ResizablePanel } from "@si/vue-lib/design-system";
import { onBeforeUnmount, onMounted, ref } from "vue";
import FuncListPanel from "@/components/FuncEditor/FuncListPanel.vue";
import FuncEditorTabs from "@/components/FuncEditor/FuncEditorTabs.vue";
import FuncDetails from "@/components/FuncEditor/FuncDetails.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import CustomizeTabs from "../CustomizeTabs.vue";

const funcStore = useFuncStore();

const leftResizablePanelRef = ref();
const rightResizablePanelRef = ref();

const onKeyDown = async (e: KeyboardEvent) => {
  if (
    e.altKey &&
    e.shiftKey &&
    leftResizablePanelRef.value &&
    rightResizablePanelRef.value
  ) {
    if (
      leftResizablePanelRef.value.collapsed &&
      rightResizablePanelRef.value.collapsed
    ) {
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
