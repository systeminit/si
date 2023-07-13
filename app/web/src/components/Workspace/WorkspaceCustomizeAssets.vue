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
            <AssetListPanel :assetId="assetId" />
          </CustomizeTabs>
        </div>
      </div>
    </template>
    <template #subpanel2>
      <AssetFuncListPanel :assetId="assetId" />
    </template>
  </ResizablePanel>
  <div
    class="grow overflow-hidden bg-shade-0 dark:bg-neutral-800 dark:text-shade-0 font-semi-bold flex flex-col relative"
  >
    <div class="left-2 right-2 top-2 bottom-2 absolute">
      <AssetEditorTabs :selectedAssetId="assetId" :selectedFuncId="funcId" />
    </div>
  </div>
  <ResizablePanel rememberSizeKey="func-details" side="right" :minSize="200">
    <div
      v-if="FF_SINGLE_MODEL_SCREEN"
      class="absolute w-full flex flex-col h-full"
    >
      <ApplyChangeSetButton class="w-10/12 m-4" />
      <SidebarSubpanelTitle v-if="assetId && !funcId"
        >Asset Details</SidebarSubpanelTitle
      >
      <SidebarSubpanelTitle v-if="assetId && funcId"
        >Asset Function Details</SidebarSubpanelTitle
      >

      <AssetDetailsPanel v-if="assetId && !funcId" :assetId="assetId" />
      <FuncDetails
        v-else-if="assetId && funcId"
        :funcId="funcId"
        :schemaVariantId="assetStore.assetsById[assetId]?.defaultVariantId"
        singleModelScreen
        @detached="onDetach"
      />
    </div>
    <template v-else>
      <AssetDetailsPanel v-if="assetId && !funcId" :assetId="assetId" />
      <FuncDetails
        v-else-if="assetId && funcId"
        :funcId="funcId"
        :schemaVariantId="assetStore.assetsById[assetId]?.defaultVariantId"
        @detached="onDetach"
      />
    </template>
  </ResizablePanel>
</template>

<script lang="ts" setup>
import { computed, onMounted, watch } from "vue";
import { useRouter } from "vue-router";
import { ResizablePanel } from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import ChangeSetPanel from "../ChangeSetPanel.vue";
import AssetListPanel from "../AssetListPanel.vue";
import CustomizeTabs from "../CustomizeTabs.vue";
import AssetEditorTabs from "../AssetEditorTabs.vue";
import AssetDetailsPanel from "../AssetDetailsPanel.vue";
import AssetFuncListPanel from "../AssetFuncListPanel.vue";
import FuncDetails from "../FuncEditor/FuncDetails.vue";

const featureFlagsStore = useFeatureFlagsStore();
const FF_SINGLE_MODEL_SCREEN = computed(
  () => featureFlagsStore.SINGLE_MODEL_SCREEN,
);

const assetStore = useAssetStore();
const router = useRouter();
const loadAssetsReqStatus = assetStore.getRequestStatus("LOAD_ASSET_LIST");

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
  // if (!assetId.value && assetStore.getLastSelectedAssetId()) {
  //   router.push({
  //     name: "workspace-lab-assets",
  //     params: {
  //       ...router.currentRoute.value.params,
  //       assetId: assetStore.getLastSelectedAssetId(),
  //     },
  //   });
  // }
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
