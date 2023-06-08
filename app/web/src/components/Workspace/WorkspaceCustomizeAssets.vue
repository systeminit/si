<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel remember-size-key="func-picker" side="left" :min-size="300">
    <div class="flex flex-col h-full">
      <div
        :style="{ height: `${topSplitSizer.height}px` }"
        class="relative flex flex-col flex-shrink-0"
      >
        <ChangeSetPanel
          class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0"
        />

        <CustomizeTabs tab-content-slug="assets">
          <AssetListPanel :asset-id="assetId" />
        </CustomizeTabs>
      </div>

      <SiPanelResizer
        panel-side="bottom"
        :style="{ top: `${topSplitSizer.height}px` }"
        class="w-full"
        @resize-start="topSplitSizer.onResizeStart"
        @resize-move="topSplitSizer.onResizeMove"
        @resize-reset="topSplitSizer.resetSize"
      />
      <div
        class="h-full border-t dark:border-neutral-600 relative z-20 p-8 dark:bg-neutral-800 bg-shade-0"
      >
        <AssetFuncListPanel />
      </div>
    </div>
  </SiPanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 text-lg font-semi-bold flex flex-col relative"
  >
    <div class="inset-2 bottom-0 absolute w-full h-full">
      <AssetEditor :asset-id="assetId" />
    </div>
  </div>
  <SiPanel remember-size-key="func-details" side="right" :min-size="200">
    <AssetDetailsPanel :asset-id="assetId" />
  </SiPanel>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { watch } from "vue";
import { useAssetStore } from "@/store/asset.store";
import SiPanelResizer, { defaultSizer } from "@/components/SiPanelResizer.vue";
import ChangeSetPanel from "../ChangeSetPanel.vue";
import SiPanel from "../SiPanel.vue";
import AssetListPanel from "../AssetListPanel.vue";
import CustomizeTabs from "../CustomizeTabs.vue";
import AssetEditor from "../AssetEditor.vue";
import AssetDetailsPanel from "../AssetDetailsPanel.vue";
import AssetFuncListPanel from "../AssetFuncListPanel.vue";

const assetStore = useAssetStore();
const loadAssetsReqStatus = assetStore.getRequestStatus("LOAD_ASSET_LIST");

const TOP_SPLIT_DEFAULT_HEIGHT = 700;
const TOP_SPLIT_MIN_HEIGHT = 150;

const topSplitSizer = defaultSizer(
  TOP_SPLIT_DEFAULT_HEIGHT,
  TOP_SPLIT_MIN_HEIGHT,
);

const props = defineProps<{
  assetId?: string;
  workspacePk: string;
  changeSetId: string;
}>();

watch(
  [() => props.assetId, loadAssetsReqStatus],
  () => {
    if (loadAssetsReqStatus.value.isSuccess && props.assetId) {
      assetStore.SELECT_ASSET(props.assetId);
    }
  },
  { immediate: true },
);
</script>
