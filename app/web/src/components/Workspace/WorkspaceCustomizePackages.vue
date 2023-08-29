<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <ResizablePanel rememberSizeKey="func-picker" side="left" :minSize="300">
    <div class="flex flex-col h-full">
      <div class="relative flex-grow">
        <CustomizeTabs tabContentSlug="packages">
          <ModuleListPanel />
        </CustomizeTabs>
      </div>
    </div>
  </ResizablePanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 font-semi-bold flex flex-col relative"
  >
    <div class="inset-0 p-sm absolute overflow-auto">
      <ModuleDisplay :key="moduleSlug" />
    </div>
  </div>
  <ResizablePanel rememberSizeKey="func-details" side="right" :minSize="200">
    <div class="flex flex-col h-full items-center">
      <ApplyChangeSetButton class="w-10/12 mx-auto my-4" />
      <SidebarSubpanelTitle>Module Details</SidebarSubpanelTitle>
      <ModuleDetailsPanel :key="moduleSlug" class="w-full" />
    </div>
  </ResizablePanel>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import { ResizablePanel } from "@si/vue-lib/design-system";
import ModuleListPanel from "@/components/modules/ModuleListPanel.vue";
import ModuleDisplay from "@/components/modules/ModuleDisplay.vue";
import ModuleDetailsPanel from "@/components/modules/ModuleDetailsPanel.vue";
import { useModuleStore } from "@/store/module.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import CustomizeTabs from "../CustomizeTabs.vue";

const moduleStore = useModuleStore();
const moduleSlug = computed(() => moduleStore.urlSelectedModuleSlug);
</script>
