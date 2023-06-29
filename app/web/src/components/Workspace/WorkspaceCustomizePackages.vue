<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel rememberSizeKey="func-picker" side="left" :minSize="300">
    <div class="flex flex-col h-full">
      <ChangeSetPanel
        class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0"
      />
      <CustomizeTabs tabContentSlug="packages">
        <ModuleListPanel />
      </CustomizeTabs>
    </div>
  </SiPanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 font-semi-bold flex flex-col relative"
  >
    <div class="inset-0 p-sm absolute overflow-auto">
      <ModuleDisplay :key="moduleSlug" />
    </div>
  </div>
  <SiPanel rememberSizeKey="func-details" side="right" :minSize="200">
    <ModuleDetailsPanel :key="moduleSlug" />
  </SiPanel>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import ChangeSetPanel from "@/components/ChangeSetPanel.vue";
import ModuleListPanel from "@/components/modules/ModuleListPanel.vue";
import ModuleDisplay from "@/components/modules/ModuleDisplay.vue";
import ModuleDetailsPanel from "@/components/modules/ModuleDetailsPanel.vue";
import { useModuleStore } from "@/store/module.store";
import SiPanel from "@/components/SiPanel.vue";
import CustomizeTabs from "../CustomizeTabs.vue";

const moduleStore = useModuleStore();
const moduleSlug = computed(() => moduleStore.urlSelectedModuleSlug);
</script>
