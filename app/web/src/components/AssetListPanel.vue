<template>
  <ScrollArea>
    <RequestStatusMessage
      v-if="loadAssetsReqStatus.isPending && assetStore.assetList.length < 1"
      :requestStatus="loadAssetsReqStatus"
      loadingMessage="Loading assets..."
    />
    <template #top>
      <div
        class="w-full p-2 border-b dark:border-neutral-600 flex gap-1 flex-row-reverse"
      >
        <VButton
          v-if="featureFlagsStore.CONTRIBUTE_BUTTON"
          label="Contribute"
          tone="action"
          icon="cloud-upload"
          size="sm"
          @click="contributeAsset"
        />
        <VButton
          label="New Asset"
          tone="action"
          icon="plus"
          size="sm"
          @click="newAsset"
        />
      </div>
      <SiSearch autoSearch placeholder="search assets" @search="onSearch" />
      <div
        class="w-full text-neutral-400 dark:text-neutral-300 text-sm text-center p-2 border-b dark:border-neutral-600"
      >
        Select an asset to view or edit it.
      </div>
    </template>
    <template v-if="assetStore.assetList.length > 0">
      <ul class="overflow-y-auto min-h-[200px]">
        <Collapsible
          v-for="category in Object.keys(categorizedAssets)"
          :key="category"
          :label="category"
          as="li"
          contentAs="ul"
          defaultOpen
          class="select-none"
        >
          <AssetListItem
            v-for="asset in categorizedAssets[category]"
            :key="asset.id"
            :a="asset"
          />
        </Collapsible>
      </ul>
    </template>
    <ModuleExportModal
      ref="contributeAssetModalRef"
      title="Contribute Assets"
      label="Contribute to System Initiative"
      :loadingText="_.sample(contributeLoadingTexts)"
      :preSelectedSchemaVariantId="assetStore.selectedAsset?.schemaVariantId"
      autoVersion
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
  Collapsible,
  ScrollArea,
  VButton,
  Modal,
  RequestStatusMessage,
} from "@si/vue-lib/design-system";
import SiSearch from "@/components/SiSearch.vue";
import { AssetListEntry, useAssetStore } from "@/store/asset.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import AssetListItem from "./AssetListItem.vue";
import ModuleExportModal from "./modules/ModuleExportModal.vue";

const assetStore = useAssetStore();
const featureFlagsStore = useFeatureFlagsStore();
const { assetList } = storeToRefs(assetStore);
const loadAssetsReqStatus = assetStore.getRequestStatus("LOAD_ASSET_LIST");
const contributeAssetModalRef = ref<InstanceType<typeof ModuleExportModal>>();
const exportSuccessModalRef = ref<InstanceType<typeof Modal>>();

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

const props = defineProps({
  assetId: { type: String },
});

const searchString = ref("");

const onSearch = (search: string) => {
  searchString.value = search.trim().toLocaleLowerCase();
};

const categorizedAssets = computed(() =>
  assetList.value
    .filter((asset) => {
      if (searchString.value.length) {
        return (
          asset.name.toLocaleLowerCase().includes(searchString.value) ||
          asset.menuName?.toLocaleLowerCase().includes(searchString.value)
        );
      }

      return true;
    })
    .reduce((categorized, asset) => {
      let catList = categorized[asset.category];
      if (!catList) {
        catList = [];
      }
      catList.push(asset);
      categorized[asset.category] = catList;
      return categorized;
    }, {} as { [key: string]: AssetListEntry[] }),
);

const newAsset = async () => {
  const result = await assetStore.CREATE_ASSET(assetStore.createNewAsset());
  if (result.result.success) {
    assetStore.selectAsset(result.result.data.id);
  }
};

const contributeAsset = () => contributeAssetModalRef.value?.open();
const onExport = () => exportSuccessModalRef.value?.open();
</script>
