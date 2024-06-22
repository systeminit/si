<template>
  <div class="flex flex-col">
    <TabGroup
      ref="tabGroupRef"
      closeable
      firstTabMarginLeft="none"
      rememberSelectedTabKey="asset-editor"
      startSelectedTabSlug="asset"
      marginTop="xs"
      @close-tab="onTabClose"
      @update:selected-tab="onTabChange"
    >
      <template #noTabs>
        <WorkspaceCustomizeEmptyState
          :requestStatus="loadAssetsRequestStatus"
          loadingMessage="Loading assets..."
          :instructions="
            assetStore.selectedSchemaVariants.length > 1
              ? 'You have selected multiple assets, use the right pane!'
              : undefined
          "
        />
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
import { TabGroup, TabGroupItem } from "@si/vue-lib/design-system";
import { useAssetStore, schemaVariantDisplayName } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import AssetEditor from "./AssetEditor.vue";
import FuncEditor from "./FuncEditor/FuncEditor.vue";
import WorkspaceCustomizeEmptyState from "./WorkspaceCustomizeEmptyState.vue";

const assetStore = useAssetStore();
const funcStore = useFuncStore();

const tabGroupRef = ref<InstanceType<typeof TabGroup>>();

const selectedAssetId = computed(() => assetStore.selectedVariantId);

const loadAssetsRequestStatus = assetStore.getRequestStatus(
  "LOAD_SCHEMA_VARIANT_LIST",
);

const currentTabs = ref<{ type: string; label: string; id: string }[]>([]);

// We have to be careful about updating this list since it will cause the entire
// tab group item list to re-render, and re-rendering will interrupt the editing
// flow
watch(
  [
    loadAssetsRequestStatus,
    () => assetStore.selectedVariantId,
    assetStore.openVariantFuncIds,
  ],
  ([requestStatus, assetId, openAssetFuncIds]) => {
    // no asset/multiple assets selected, don't show tabs
    if (!assetId) {
      currentTabs.value = [];
      return;
    }
    if (!requestStatus.isSuccess) {
      return;
    }
    const asset = assetStore.variantFromListById[assetId];
    const assetTab = {
      type: "asset",
      label: asset ? schemaVariantDisplayName(asset) ?? "error" : "error",
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

    if (funcStore.selectedFuncId)
      tabGroupRef.value?.selectTab(funcStore.selectedFuncId);

    // still dont know what is racing and removing the querystring from the URL
    setTimeout(() => {
      if (assetStore.selectedVariantId)
        assetStore.setSchemaVariantSelection(assetStore.selectedVariantId);
      funcTabs.forEach((f) => {
        assetStore.addFuncSelection(f.id);
      });
    }, 100); // wait until after the querystring is stripped, and rebuild it
  },
  { immediate: true },
);

watch(
  () => funcStore.selectedFuncId,
  () => {
    if (funcStore.selectedFuncId) {
      tabGroupRef.value?.selectTab(funcStore.selectedFuncId);
    }
  },
);

const onTabChange = async (tabSlug: string | undefined) => {
  if (tabSlug !== "asset") {
    funcStore.selectedFuncId = tabSlug;
  } else {
    funcStore.selectedFuncId = undefined;
  }
};

const onTabClose = (funcId: string) => {
  assetStore.removeFuncSelection(funcId);
  if (selectedAssetId.value && typeof selectedAssetId.value === "string") {
    assetStore.closeFunc(selectedAssetId.value, funcId);
  }
};

const closeTab = (slug: string) => {
  if (tabGroupRef.value) {
    tabGroupRef.value.closeTabBySlug(slug);
    funcStore.selectedFuncId =
      tabGroupRef.value.selectedTabSlug !== "asset"
        ? tabGroupRef.value.selectedTabSlug
        : undefined;
  }
  assetStore.removeFuncSelection(slug);
};
</script>
