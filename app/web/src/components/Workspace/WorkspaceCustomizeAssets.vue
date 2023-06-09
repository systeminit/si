<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel remember-size-key="func-picker" side="left" :min-size="300">
    <div class="flex flex-col h-full">
      <ChangeSetPanel
        class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0"
      />
      <CustomizeTabs tab-content-slug="assets">
        <AssetListPanel :asset-id="assetId" />
      </CustomizeTabs>
    </div>
  </SiPanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 font-semi-bold flex flex-col relative"
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
import ChangeSetPanel from "../ChangeSetPanel.vue";
import SiPanel from "../SiPanel.vue";
import AssetListPanel from "../AssetListPanel.vue";
import CustomizeTabs from "../CustomizeTabs.vue";
import AssetEditor from "../AssetEditor.vue";
import AssetDetailsPanel from "../AssetDetailsPanel.vue";

const assetStore = useAssetStore();
const loadAssetsReqStatus = assetStore.getRequestStatus("LOAD_ASSET_LIST");

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
