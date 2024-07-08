<template>
  <ScrollArea>
    <RequestStatusMessage
      v-if="loadAssetsReqStatus.isPending && assetStore.variantList.length < 1"
      :requestStatus="loadAssetsReqStatus"
      loadingMessage="Loading assets..."
    />
    <template #top>
      <SidebarSubpanelTitle icon="component">
        <template #label>
          <div class="flex flex-row gap-xs">
            <div>Assets</div>
            <PillCounter :count="assetList.length" />
          </div>
        </template>
        <div class="flex flex-row gap-xs items-center">
          <IconButton
            :requestStatus="createAssetReqStatus"
            class="hover:scale-125"
            icon="plus"
            loadingIcon="loader"
            loadingTooltip="Creating Asset..."
            size="sm"
            tooltip="New Asset"
            tooltipPlacement="top"
            variant="simple"
            @click="() => newAssetModalRef?.modal?.open()"
          />
          <IconButton
            v-if="canContribute || true"
            :selected="contributeAssetModalRef?.isOpen || false"
            class="hover:scale-125"
            icon="cloud-upload"
            size="sm"
            tooltip="Contribute All"
            tooltipPlacement="top"
            variant="simple"
            @click="contributeAsset"
          />
          <IconButton
            v-if="canUpdate"
            class="hover:scale-125"
            icon="code-deployed"
            size="sm"
            tooltip="Update All"
            tooltipPlacement="top"
            variant="simple"
          />
        </div>
        <AssetNameModal
          ref="newAssetModalRef"
          :loading="createAssetReqStatus.isPending"
          buttonLabel="Create Asset"
          title="New Asset"
          @submit="(name) => newAsset(name)"
        />
      </SidebarSubpanelTitle>
      <SiSearch
        ref="searchRef"
        :filters="searchFiltersWithCounts"
        placeholder="search assets"
        @search="onSearch"
      />
      <!-- <div
        class="w-full text-neutral-400 dark:text-neutral-300 text-sm text-center p-xs border-b dark:border-neutral-600"
      >
        Select an asset to view or edit it.
      </div> -->
    </template>
    <template v-if="assetStore.variantList.length > 0">
      <TreeNode
        v-for="category in Object.keys(categorizedAssets)"
        :key="category"
        :color="categoryColor(category)"
        :label="category"
        :primaryIcon="getAssetIcon(category)"
        alwaysShowArrow
        clickLabelToToggle
        enableDefaultHoverClasses
        enableGroupToggle
        indentationSize="none"
      >
        <template #icons>
          <PillCounter
            :count="categorizedAssets[category]?.length || 0"
            showHoverInsideTreeNode
          />
        </template>
        <AssetListItem
          v-for="asset in categorizedAssets[category]"
          :key="asset.schemaVariantId"
          :a="asset"
          :c="categorizedAssets[category]"
        />
      </TreeNode>
    </template>
    <ModuleExportModal
      ref="contributeAssetModalRef"
      :loadingText="_.sample(contributeLoadingTexts)"
      :preSelectedSchemaVariantId="
        assetStore.selectedSchemaVariant?.schemaVariantId
      "
      label="Contribute to System Initiative"
      title="Contribute Assets"
      @export-success="onExport"
    />
    <Modal ref="exportSuccessModalRef" size="sm" title="Contribution sent">
      <p>
        Thanks for contributing! We will review your contribution, and reach out
        via email or on Discord if we have any questions.
      </p>
      <p class="text-right">Best,</p>
      <p class="text-right">The System Initiative Developers</p>
    </Modal>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import {
  ScrollArea,
  Modal,
  RequestStatusMessage,
  TreeNode,
  PillCounter,
} from "@si/vue-lib/design-system";
import SiSearch, { Filter } from "@/components/SiSearch.vue";
import { SchemaVariantListEntry, useAssetStore } from "@/store/asset.store";
import { getAssetIcon } from "@/store/components.store";
import AssetNameModal from "./AssetNameModal.vue";
import AssetListItem from "./AssetListItem.vue";
import ModuleExportModal from "./modules/ModuleExportModal.vue";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";
import IconButton from "./IconButton.vue";

const assetStore = useAssetStore();
const { variantList: assetList } = storeToRefs(assetStore);
const loadAssetsReqStatus = assetStore.getRequestStatus(
  "LOAD_SCHEMA_VARIANT_LIST",
);
const createAssetReqStatus = assetStore.getRequestStatus("CREATE_VARIANT");
const contributeAssetModalRef = ref<InstanceType<typeof ModuleExportModal>>();
const exportSuccessModalRef = ref<InstanceType<typeof Modal>>();
const newAssetModalRef = ref<InstanceType<typeof AssetNameModal>>();

const contributeLoadingTexts = [
  "Engaging Photon Torpedos...",
  "Reticulating Splines...",
  "Revolutionizing DevOps...",
  "Calibrating Hyperspace Matrix...",
  "Syncing Neural Circuitry...",
  "Optimizing Tachyon Weave...",
  "Tuning Fractal Harmonics...",
  "Reshuffling Multiverse Threads...",
  "Harmonizing Subspace Arrays...",
  "Modulating Cybernetic Matrices...",
  "Configuring Exo-Geometric Arrays...",
  "Initializing Flux Capacitors...",
  "Balancing Subatomic Resonance...",
  "Fine-tuning Quantum Entanglement...",
  "Matrixing Hyperdimensional Grids...",
  "Coalescing Esoteric Code...",
  "Syncopating Quantum Flux...",
  "Reformatting Reality Lattice...",
  "Fine-tuning Temporal Flux...",
  "Syncing Cosmic Harmonics...",
];

const searchRef = ref<InstanceType<typeof SiSearch>>();
const searchString = ref("");

const onSearch = (search: string) => {
  searchString.value = search.trim().toLocaleLowerCase();
};

const canContribute = computed(() =>
  assetList.value.some((a) => a.canContribute),
);
const canUpdate = computed(() => assetList.value.some((a) => a.canUpdate));

const categorizedAssets = computed(() =>
  assetList.value
    .filter((asset) => {
      let include = true;

      if (
        searchRef.value?.filteringActive &&
        searchRef.value?.activeFilters.filter(Boolean).length > 0
      ) {
        const idxs = searchRef.value?.activeFilters.flatMap((bool, idx) =>
          bool ? idx : [],
        );
        include = false;
        idxs.forEach((idx) => {
          if (filters.value[idx]?.includes(asset)) {
            include = true;
          }
        });
      }

      if (include && searchString.value.length) {
        include = !!(
          asset.schemaName.toLocaleLowerCase().includes(searchString.value) ||
          asset.displayName?.toLocaleLowerCase().includes(searchString.value) ||
          asset.category?.toLocaleLowerCase().includes(searchString.value)
        );
      }

      return include;
    })
    .reduce((categorized, asset) => {
      let catList = categorized[asset.category];
      if (!catList) {
        catList = [];
      }
      catList.push(asset);
      categorized[asset.category] = catList;
      return categorized;
    }, {} as { [key: string]: SchemaVariantListEntry[] }),
);

const categoryColor = (category: string) => {
  const assets = categorizedAssets.value[category];

  if (assets && assets[0]) {
    return assets[0].color;
  }

  return "#000";
};

const newAsset = async (newAssetName: string) => {
  const result = await assetStore.CREATE_VARIANT(newAssetName);
  if (result.result.success) {
    assetStore.setSchemaVariantSelection(result.result.data.schemaVariantId);
    newAssetModalRef.value?.modal?.close();
  } else if (result.result.statusCode === 409) {
    newAssetModalRef.value?.setError("That name is already in use");
  }
  newAssetModalRef.value?.reset();
};

const contributeAsset = () => contributeAssetModalRef.value?.open();
const onExport = () => exportSuccessModalRef.value?.open();

const filters = computed(() => [
  assetList.value.filter((a) => a.canContribute),
  assetList.value.filter((a) => a.canUpdate),
]);

const searchFiltersWithCounts = computed(() => {
  const searchFilters: Array<Filter> = [
    {
      name: "Assets to Contribute",
      iconTone: "action",
      iconName: "cloud-upload",
      count: filters.value[0]?.length,
    },
    {
      name: "Updates Available",
      iconTone: "action",
      iconName: "code-deployed",
      count: filters.value[1]?.length,
    },
  ];
  return searchFilters;
});
</script>
