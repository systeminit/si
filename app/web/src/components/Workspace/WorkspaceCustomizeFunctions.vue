<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel rememberSizeKey="func-picker" side="left" :minSize="300">
    <div class="flex flex-col h-full">
      <ChangeSetPanel v-if="!FF_SINGLE_MODEL_SCREEN" />

      <CustomizeTabs tabContentSlug="functions">
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
  <SiPanel rememberSizeKey="func-details" side="right" :minSize="200">
    <div
      v-if="FF_SINGLE_MODEL_SCREEN"
      class="absolute w-full flex flex-col h-full"
    >
      <ApplyChangeSetButton class="w-10/12 m-4" />
      <SidebarSubpanelTitle>Function Details</SidebarSubpanelTitle>

      <FuncDetails
        :key="funcStore.urlSelectedFuncId"
        :funcId="funcStore.urlSelectedFuncId"
        singleModelScreen
      />
    </div>
    <FuncDetails
      v-else
      :key="funcStore.urlSelectedFuncId"
      :funcId="funcStore.urlSelectedFuncId"
    />
  </SiPanel>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import ChangeSetPanel from "@/components/ChangeSetPanel.vue";
import FuncListPanel from "@/components/FuncEditor/FuncListPanel.vue";
import FuncEditorTabs from "@/components/FuncEditor/FuncEditorTabs.vue";
import FuncDetails from "@/components/FuncEditor/FuncDetails.vue";
import SiPanel from "@/components/SiPanel.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import CustomizeTabs from "../CustomizeTabs.vue";

const featureFlagsStore = useFeatureFlagsStore();
const FF_SINGLE_MODEL_SCREEN = computed(
  () => featureFlagsStore.SINGLE_MODEL_SCREEN,
);

const funcStore = useFuncStore();
</script>
