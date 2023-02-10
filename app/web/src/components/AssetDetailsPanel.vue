<template>
  <div>
    <RequestStatusMessage
      v-if="loadAssetsReqStatus.isPending"
      :request-status="loadAssetsReqStatus"
      show-loader-without-message
    />
    <div v-else-if="assetStore.selectedAsset && assetId" class="flex flex-col">
      <div
        class="p-sm border-b dark:border-neutral-600 flex flex-row items-center justify-between"
      >
        <NodeSkeleton :color="assetStore.selectedAsset.color" size="mini" />
        <div class="font-bold truncate leading-relaxed">
          {{ assetDisplayName(assetStore.selectedAsset) }}
        </div>
        <VButton2
          label="Execute"
          tone="action"
          icon="plus"
          size="md"
          @click="executeAsset"
        />
      </div>
      <div class="p-sm flex flex-col">
        <div class="pb-xs font-bold text-xl">Name:</div>
        <div class="text-md">
          {{ assetDisplayName(assetStore.selectedAsset) }}
        </div>
      </div>
      <div class="p-sm flex flex-col">
        <div class="pb-xs font-bold text-xl">Category:</div>
        <div class="text-md">{{ assetStore.selectedAsset.category }}</div>
      </div>
      <div class="p-sm flex flex-col">
        <div class="pb-xs font-bold text-xl">Description:</div>
        <div class="text-md">{{ assetStore.selectedAsset.description }}</div>
      </div>
      <div class="p-sm flex flex-col">
        <div class="pb-xs font-bold text-xl">Color:</div>
        <div
          class="text-md"
          :style="`color: #${assetStore.selectedAsset.color}`"
        >
          #{{ assetStore.selectedAsset.color }}
        </div>
      </div>
      <div class="p-sm flex flex-col">
        <div class="pb-xs font-bold text-xl">Documentation:</div>
        <div class="text-md text-action-500 font-bold">
          <a :href="assetStore.selectedAsset.link" target="_blank">
            Documentation Link
          </a>
        </div>
      </div>
    </div>
    <div
      v-else
      class="px-2 py-sm text-center text-neutral-400 dark:text-neutral-300"
    >
      <template v-if="assetId">Asset "{{ assetId }}" does not exist!</template>
      <template v-else>Select an asset to view its details.</template>
    </div>
    <Modal ref="executeAssetModalRef" size="sm" :title="assetModalTitle">
      The asset you just created will now appear in the Assets Panel.
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import { useAssetStore, assetDisplayName } from "@/store/asset.store";
import RequestStatusMessage from "@/ui-lib/RequestStatusMessage.vue";
import Modal from "@/ui-lib/modals/Modal.vue";
import NodeSkeleton from "./NodeSkeleton.vue";

const assetStore = useAssetStore();
const loadAssetsReqStatus = assetStore.getRequestStatus("LOAD_ASSET_LIST");
const executeAssetModalRef = ref();
const assetModalTitle = ref("New Asset Created");

defineProps<{
  assetId?: string;
}>();

const executeAsset = () => {
  executeAssetModalRef.value.open();
};
</script>
