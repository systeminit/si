<template>
  <div class="flex flex-col">
    <TabGroup
      ref="tabGroupRef"
      closeable
      firstTabMarginLeft="none"
      rememberSelectedTabKey="asset-editor"
      startSelectedTabSlug="asset"
      @close-tab="onTabClose"
      @update:selected-tab="onTabChange"
    >
      <template #noTabs>
        <div class="text-center text-neutral-400 dark:text-neutral-300">
          <RequestStatusMessage
            v-if="loadAssetsRequestStatus.isPending"
            :requestStatus="loadAssetsRequestStatus"
            loadingMessage="Loading assets..."
          />
          <template v-else-if="loadAssetsRequestStatus.isSuccess">
            <div class="text-center p-lg">Select an asset to edit it.</div>
          </template>
        </div>
      </template>
      <TabGroupItem
        v-for="tab in currentTabs"
        :key="tab.type === 'asset' ? 'asset' : tab.id"
        :uncloseable="tab.type === 'asset'"
        :label="tab.label"
        :slug="tab.type === 'asset' ? 'asset' : tab.id"
      >
        <AssetEditor
          v-if="tab.type === 'asset'"
          :key="selectedAssetId"
          :assetId="selectedAssetId"
        />

        <FuncEditor
          v-else-if="tab.type === 'func'"
          :key="tab.id"
          :funcId="tab.id"
          @close="closeTab(tab.id)"
        />
      </TabGroupItem>
    </TabGroup>
  </div>
</template>

<script lang="ts" setup>
import isEqual from "lodash-es/isEqual";
import { watch, ref, computed } from "vue";
import {
  TabGroup,
  TabGroupItem,
  RequestStatusMessage,
} from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import AssetEditor from "./AssetEditor.vue";
import FuncEditor from "./FuncEditor/FuncEditor.vue";

const assetStore = useAssetStore();
const funcStore = useFuncStore();

const tabGroupRef = ref<InstanceType<typeof TabGroup>>();

const selectedFuncId = computed(() => assetStore.selectedFuncId);
const selectedAssetId = computed(() => assetStore.selectedAssetId);

const loadAssetsRequestStatus = assetStore.getRequestStatus("LOAD_ASSET_LIST");

const currentTabs = ref<{ type: string; label: string; id: string }[]>([]);

// We have to be careful about updating this list since it will cause the entire
// tab group item list to re-render, and re-rendering will interrupt the editing
// flow
watch(
  [
    loadAssetsRequestStatus,
    () => assetStore.selectedAssetId,
    assetStore.openAssetFuncIds,
  ],
  ([requestStatus, assetId, openAssetFuncIds]) => {
    if (!requestStatus.isSuccess || !assetId) {
      return;
    }

    const assetTab = {
      type: "asset",
      label: assetStore.assetListEntryById(assetId)?.name ?? "error",
      id: assetId,
    };

    const funcTabs =
      openAssetFuncIds[assetId]?.map((funcId) => ({
        type: "func",
        label: funcStore.funcsById[funcId]?.name ?? "error",
        id: funcId,
      })) ?? [];

    const tabs = [assetTab].concat(funcTabs);
    if (!isEqual(tabs, currentTabs.value)) {
      // updating tab list
      currentTabs.value = tabs;
    }
  },
);

const onTabChange = async (tabSlug: string | undefined) => {
  // tabSlugs are just func ids here, besides the asset tab, which is just "asset"
  if (tabSlug === "asset") {
    tabSlug = undefined;
  } else if (!tabSlug || tabSlug === selectedFuncId.value) {
    return;
  }

  assetStore.selectAsset(assetStore.urlSelectedAssetId, tabSlug);
};

const onTabClose = (funcId: string) => {
  if (selectedAssetId.value && typeof selectedAssetId.value === "string") {
    assetStore.closeFunc(selectedAssetId.value, funcId);
  }
};

const closeTab = (slug: string) => {
  if (tabGroupRef.value) {
    tabGroupRef.value.closeTabBySlug(slug);
  }
};

const loadFuncDetailsReqStatus = funcStore.getRequestStatus(
  "FETCH_FUNC_DETAILS",
  assetStore.urlSelectedFuncId,
);

watch([() => assetStore.selectedFuncId, loadFuncDetailsReqStatus], () => {
  if (
    assetStore.selectedAssetId &&
    !assetStore.selectedFuncId &&
    loadFuncDetailsReqStatus.value.isSuccess
  ) {
    tabGroupRef.value?.selectTab("asset");
  } else if (assetStore.selectedAssetId && assetStore.selectedFuncId) {
    tabGroupRef.value?.selectTab(assetStore.selectedFuncId);
  }
});
</script>
