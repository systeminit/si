<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel remember-size-key="func-picker" side="left" :min-size="300">
    <div class="flex flex-col h-full">
      <ChangeSetPanel />

      <CustomizeTabs tab-content-slug="functions">
        <FuncListPanel />
      </CustomizeTabs>
    </div>
  </SiPanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 font-semi-bold flex flex-col relative"
  >
    <div class="left-2 right-2 top-2 bottom-2 absolute">
      <FuncEditorTabs />
    </div>
  </div>
  <SiPanel remember-size-key="func-details" side="right" :min-size="200">
    <FuncDetails :key="funcStore.urlSelectedFuncId" />
  </SiPanel>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import ChangeSetPanel from "@/components/ChangeSetPanel.vue";
import FuncListPanel from "@/components/FuncEditor/FuncListPanel.vue";
import FuncEditorTabs from "@/components/FuncEditor/FuncEditorTabs.vue";
import FuncDetails from "@/components/FuncEditor/FuncDetails.vue";
import SiPanel from "@/components/SiPanel.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import CustomizeTabs from "../CustomizeTabs.vue";

const funcStore = useFuncStore();
</script>
