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
        <div class="p-2 text-center text-neutral-400 dark:text-neutral-300">
          <RequestStatusMessage
            v-if="loadAssetsRequestStatus.isPending"
            :requestStatus="loadAssetsRequestStatus"
            showLoaderWithoutMessage
          />
          <template v-else-if="loadAssetsRequestStatus.isSuccess">
            Select an asset, to edit it...
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
        />
      </TabGroupItem>
    </TabGroup>
  </div>
</template>

<script lang="ts" setup>
import isEqual from "lodash-es/isEqual";
import { watch, ref, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
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
const route = useRoute();
const router = useRouter();

const tabGroupRef = ref<InstanceType<typeof TabGroup>>();

const selectedFuncId = computed(() => assetStore.urlSelectedFuncId);
const selectedAssetId = computed(() => assetStore.urlSelectedAssetId);

const loadAssetsRequestStatus = assetStore.getRequestStatus("LOAD_ASSET_LIST");

const currentTabs = ref<{ type: string; label: string; id: string }[]>([]);

// We have to be careful about updating this list since it will cause the entire
// tab group item list to re-render, and re-rendering will interrupt the editing
// flow
watch(
  [
    loadAssetsRequestStatus,
    () => assetStore.urlSelectedAssetId,
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

const onTabChange = (tabSlug: string | undefined) => {
  if (!tabSlug) {
    return;
  }
  // tabSlugs are just func ids here, besides the asset tab, which is just
  // "asset"
  if (tabSlug === "asset") {
    router.replace({
      name: "workspace-lab-assets",
      params: {
        ...route.params,
        assetId: assetStore.urlSelectedAssetId,
        funcId: undefined,
      },
    });
  } else if (tabSlug !== selectedFuncId.value) {
    router.replace({
      name: "workspace-lab-assets",
      params: {
        ...route.params,
        assetId: assetStore.urlSelectedAssetId,
        funcId: tabSlug,
      },
    });
  }
};

const onTabClose = (funcId: string) => {
  if (selectedAssetId.value && typeof selectedAssetId.value === "string") {
    assetStore.closeFunc(selectedAssetId.value, funcId);
  }
};

watch(
  [() => assetStore.urlSelectedAssetId, () => assetStore.urlSelectedFuncId],
  () => {
    if (assetStore.urlSelectedAssetId && !assetStore.urlSelectedFuncId) {
      tabGroupRef.value?.selectTab("asset");
    } else if (assetStore.urlSelectedAssetId && assetStore.urlSelectedFuncId) {
      tabGroupRef.value?.selectTab(assetStore.urlSelectedFuncId);
    }
  },
);
</script>
