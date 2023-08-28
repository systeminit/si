<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <ResizablePanel rememberSizeKey="func-picker" side="left" :minSize="300">
    <template #subpanel1>
      <div class="flex flex-col h-full">
        <ChangeSetPanel
          v-if="!FF_SINGLE_MODEL_SCREEN"
          class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0"
        />

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
  <ResizablePanel rememberSizeKey="func-details" side="right" :minSize="200">
    <div
      v-if="FF_SINGLE_MODEL_SCREEN"
      class="absolute w-full flex flex-col h-full"
    >
      <div
        v-if="!changeSetsStore.headSelected"
        class="p-xs border-b dark:border-neutral-500"
      >
        <ApplyChangeSetButton class="w-full" />
      </div>
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
          @detached="onDetach"
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
    <template v-else>
      <AssetDetailsPanel
        v-if="assetStore.selectedAssetId && !assetStore.selectedFuncId"
        :key="assetStore.selectedAssetId"
        :assetId="assetStore.selectedAssetId"
      />
      <FuncDetails
        v-else-if="assetStore.selectedAssetId && assetStore.selectedFuncId"
        :funcId="assetStore.selectedFuncId"
        :schemaVariantId="assetStore.selectedAsset?.schemaVariantId"
        @detached="onDetach"
      />
    </template>
  </ResizablePanel>
</template>

<script lang="ts" setup>
import { computed, watch } from "vue";
import { ResizablePanel } from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import ChangeSetPanel from "../ChangeSetPanel.vue";
import AssetListPanel from "../AssetListPanel.vue";
import CustomizeTabs from "../CustomizeTabs.vue";
import AssetEditorTabs from "../AssetEditorTabs.vue";
import AssetDetailsPanel from "../AssetDetailsPanel.vue";
import AssetFuncListPanel from "../AssetFuncListPanel.vue";
import FuncDetails from "../FuncEditor/FuncDetails.vue";

const funcStore = useFuncStore();
const featureFlagsStore = useFeatureFlagsStore();
const FF_SINGLE_MODEL_SCREEN = computed(
  () => featureFlagsStore.SINGLE_MODEL_SCREEN,
);

const changeSetsStore = useChangeSetsStore();

const assetStore = useAssetStore();
const loadAssetsReqStatus = assetStore.getRequestStatus("LOAD_ASSET_LIST");

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
