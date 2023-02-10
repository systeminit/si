<template>
  <RequestStatusMessage
    v-if="loadAssetReqStatus.isPending"
    :request-status="loadAssetReqStatus"
    show-loader-without-message
  />
  <div
    v-else-if="
      assetStore.selectedAsset && assetStore.selectedAsset.id === assetId
    "
    class="p-sm flex flex-col h-full"
  >
    <div class="flex flex-row items-center gap-2 flex-none pb-sm">
      <NodeSkeleton :color="`#${assetStore.selectedAsset.color}`" />
      <div class="text-3xl font-bold truncate">
        {{ assetDisplayName(assetStore.selectedAsset) }}
      </div>
    </div>
    <div
      class="text-sm italic pb-sm flex flex-row flex-wrap gap-x-8 gap-y-1 flex-none"
    >
      <div>
        <span class="font-bold">Created At: </span>
        <Timestamp :date="assetStore.selectedAsset.createdAt" size="long" />
      </div>
      <div><span class="font-bold">Created By: </span>sally@systeminit.com</div>
    </div>
    <!-- TODO(wendy) - this should be a code editor and not just a viewer -->
    <CodeViewer :code="assetStore.selectedAsset.definition">
      <template #title>
        <div class="truncate">
          Code for "{{ assetDisplayName(assetStore.selectedAsset) }}"
        </div>
      </template>
    </CodeViewer>
  </div>
  <div v-else class="p-2 text-center text-neutral-400 dark:text-neutral-300">
    <template v-if="assetId">Asset "{{ assetId }}" does not exist!</template>
    <template v-else>Select an asset to view it.</template>
  </div>
</template>

<script lang="ts" setup>
import { useAssetStore, assetDisplayName } from "@/store/asset.store";
import RequestStatusMessage from "@/ui-lib/RequestStatusMessage.vue";
import Timestamp from "@/ui-lib/Timestamp.vue";
import CodeViewer from "./CodeViewer.vue";
import NodeSkeleton from "./NodeSkeleton.vue";

const assetStore = useAssetStore();
const loadAssetReqStatus = assetStore.getRequestStatus("LOAD_ASSET");

defineProps<{
  assetId?: string;
}>();
</script>
