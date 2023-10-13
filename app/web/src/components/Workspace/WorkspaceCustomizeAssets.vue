<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <ResizablePanel rememberSizeKey="func-picker" side="left" :minSize="300">
    <template #subpanel1>
      <div class="flex flex-col h-full">
        <div class="relative flex-grow">
          <CustomizeTabs tabContentSlug="assets">
            <AssetListPanel :assetId="assetStore.selectedAssetId" />
          </CustomizeTabs>
        </div>
      </div>
    </template>
    <template #subpanel2>
      <AssetFuncListPanel :assetId="assetStore.selectedAssetId" />
    </template>
  </ResizablePanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 font-semi-bold flex flex-col relative"
  >
    <div class="left-2 right-2 top-0 bottom-2 absolute">
      <AssetEditorTabs
        :selectedAssetId="assetStore.selectedAssetId"
        :selectedFuncId="assetStore.selectedFuncId"
      />
    </div>
  </div>
  <ResizablePanel
    ref="rightPanelRef"
    rememberSizeKey="func-details"
    side="right"
    :minSize="200"
  >
    <div class="absolute w-full flex flex-col h-full">
      <SidebarSubpanelTitle>
        {{
          assetStore.selectedFuncId ? "Asset Function Details" : "Asset Details"
        }}
      </SidebarSubpanelTitle>
      <template v-if="assetStore.selectedAssetId">
        <FuncDetails
          v-if="assetStore.selectedFuncId"
          :funcId="assetStore.selectedFuncId"
          :schemaVariantId="assetStore.selectedAsset?.schemaVariantId"
          singleModelScreen
          testPanelEnabled
          @detached="onDetach"
          @expand-panel="rightPanelRef.maximize"
        />
        <!-- the key here is to force remounting so we get the proper asset
        request statuses -->
        <AssetDetailsPanel
          v-else
          :key="assetStore.selectedAssetId"
          :assetId="assetStore.selectedAssetId"
        />
      </template>
      <div
        v-else
        class="p-sm text-center text-neutral-400 dark:text-neutral-300"
      >
        Select an asset to edit it.
      </div>
    </div>
  </ResizablePanel>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import { ResizablePanel } from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import AssetListPanel from "../AssetListPanel.vue";
import CustomizeTabs from "../CustomizeTabs.vue";
import AssetEditorTabs from "../AssetEditorTabs.vue";
import AssetDetailsPanel from "../AssetDetailsPanel.vue";
import AssetFuncListPanel from "../AssetFuncListPanel.vue";
import FuncDetails from "../FuncEditor/FuncDetails.vue";

const funcStore = useFuncStore();

const assetStore = useAssetStore();
const loadAssetsReqStatus = assetStore.getRequestStatus("LOAD_ASSET_LIST");

const rightPanelRef = ref();

watch(
  [
    () => assetStore.urlSelectedAssetId,
    () => assetStore.urlSelectedFuncId,
    loadAssetsReqStatus,
  ],
  async () => {
    if (loadAssetsReqStatus.value.isSuccess && assetStore.urlSelectedAssetId) {
      if (assetStore.urlSelectedAssetId && !assetStore.selectedAsset) {
        await assetStore.LOAD_ASSET(assetStore.urlSelectedAssetId);
      }

      if (assetStore.urlSelectedFuncId && !assetStore.selectedFunc) {
        await funcStore.FETCH_FUNC_DETAILS(assetStore.urlSelectedFuncId);
      }

      if (assetStore.selectedAssetId && assetStore.selectedFuncId) {
        assetStore.openFunc(
          assetStore.selectedAssetId,
          assetStore.selectedFuncId,
        );
      }
    }
  },
  { immediate: true },
);

const onDetach = async () => {
  if (assetStore.selectedAssetId && assetStore.selectedFuncId) {
    assetStore.closeFunc(assetStore.selectedAssetId, assetStore.selectedFuncId);
  }
};
</script>
