<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel rememberSizeKey="func-picker" side="left" :minSize="300">
    <div class="flex flex-col h-full">
      <div
        :style="{ height: `${topSplitSizer.height}px` }"
        class="relative flex flex-col flex-shrink-0"
      >
        <ChangeSetPanel
          class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0"
        />

        <CustomizeTabs tabContentSlug="assets">
          <AssetListPanel :assetId="assetId" />
        </CustomizeTabs>
      </div>

      <SiPanelResizer
        panelSide="bottom"
        :style="{ top: `${topSplitSizer.height}px` }"
        class="w-full"
        @resize-start="topSplitSizer.onResizeStart"
        @resize-move="topSplitSizer.onResizeMove"
        @resize-reset="topSplitSizer.resetSize"
      />
      <div
        class="h-full border-t dark:border-neutral-600 relative z-20 p-8 dark:bg-neutral-800 bg-shade-0"
      >
        <AssetFuncListPanel :assetId="assetId" />
      </div>
    </div>
  </SiPanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 font-semi-bold flex flex-col relative"
  >
    <div class="left-2 right-2 top-2 bottom-2 absolute">
      <AssetEditorTabs :selectedAssetId="assetId" :selectedFuncId="funcId" />
    </div>
  </div>
  <SiPanel rememberSizeKey="func-details" side="right" :minSize="200">
    <AssetDetailsPanel v-if="assetId && !funcId" :assetId="assetId" />
    <FuncDetails
      v-else-if="assetId && funcId"
      :funcId="funcId"
      :schemaVariantId="assetStore.assetsById[assetId]?.defaultVariantId"
      @detached="onDetach"
    />
  </SiPanel>
</template>

<script lang="ts" setup>
import { computed, onMounted, watch } from "vue";
import { useRouter } from "vue-router";
import { useAssetStore } from "@/store/asset.store";
import SiPanelResizer, { defaultSizer } from "@/components/SiPanelResizer.vue";
import ChangeSetPanel from "../ChangeSetPanel.vue";
import SiPanel from "../SiPanel.vue";
import AssetListPanel from "../AssetListPanel.vue";
import CustomizeTabs from "../CustomizeTabs.vue";
import AssetEditorTabs from "../AssetEditorTabs.vue";
import AssetDetailsPanel from "../AssetDetailsPanel.vue";
import AssetFuncListPanel from "../AssetFuncListPanel.vue";
import FuncDetails from "../FuncEditor/FuncDetails.vue";

const assetStore = useAssetStore();
const router = useRouter();
const loadAssetsReqStatus = assetStore.getRequestStatus("LOAD_ASSET_LIST");

const TOP_SPLIT_DEFAULT_HEIGHT = 700;
const TOP_SPLIT_MIN_HEIGHT = 150;

const topSplitSizer = defaultSizer(
  TOP_SPLIT_DEFAULT_HEIGHT,
  TOP_SPLIT_MIN_HEIGHT,
);

const assetId = computed(() => assetStore.urlSelectedAssetId);
const funcId = computed(() => assetStore.urlSelectedFuncId);

watch([assetId, funcId], () => {
  if (funcId.value && assetId.value) {
    assetStore.SELECT_FUNC(assetId.value, funcId.value);
  }
});

watch(
  [assetId, funcId, loadAssetsReqStatus],
  () => {
    if (loadAssetsReqStatus.value.isSuccess && assetId.value && !funcId.value) {
      assetStore.SELECT_ASSET(assetId.value);
    }
  },
  { immediate: true },
);

onMounted(async () => {
  if (!assetId.value && assetStore.getLastSelectedAssetId()) {
    router.push({
      name: "workspace-lab-assets",
      params: {
        ...router.currentRoute.value.params,
        assetId: assetStore.getLastSelectedAssetId(),
      },
    });
  }
});

const onDetach = async () => {
  if (assetStore.urlSelectedAssetId) {
    await assetStore.LOAD_ASSET(assetStore.urlSelectedAssetId);
    router.push({
      name: "workspace-lab-assets",
      params: {
        ...router.currentRoute.value.params,
        funcId: assetStore.urlSelectedFuncId,
        assetId: assetStore.urlSelectedAssetId,
      },
    });
  }
};
</script>
