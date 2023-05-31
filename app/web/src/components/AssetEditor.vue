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
      <!-- TODO: Populate the created by from SDF actorHistory-->
      <div><span class="font-bold">Created By: </span>System Initiative</div>
      <SiChip
        v-if="assetStore.selectedAsset.variantExists"
        variant="warning"
        text="read-only"
      />
    </div>
    <div class="flex-grow relative overflow-auto">
      <CodeEditor
        v-model="editingAsset"
        json
        :disabled="assetStore.selectedAsset.variantExists"
        @change="onChange"
      />
    </div>
  </div>
  <div v-else class="p-2 text-center text-neutral-400 dark:text-neutral-300">
    <template v-if="assetId">Asset "{{ assetId }}" does not exist!</template>
    <template v-else>Select an asset to view it.</template>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { Timestamp, RequestStatusMessage } from "@si/vue-lib/design-system";
import { useAssetStore, assetDisplayName } from "@/store/asset.store";
import SiChip from "@/components/SiChip.vue";
import CodeEditor from "./CodeEditor.vue";
import NodeSkeleton from "./NodeSkeleton.vue";

const assetStore = useAssetStore();
const { selectedAsset } = storeToRefs(assetStore);
const loadAssetReqStatus = assetStore.getRequestStatus("LOAD_ASSET");

const editingAsset = ref<string>(selectedAsset.value?.definition ?? "");

defineProps<{
  assetId?: string;
}>();

watch(
  () => selectedAsset.value,
  async (selectedAsset) => {
    if (editingAsset.value !== selectedAsset?.definition) {
      editingAsset.value = selectedAsset?.definition ?? "";
    }
  },
  { immediate: true },
);

const onChange = () => {
  if (
    !selectedAsset.value ||
    selectedAsset.value.definition === editingAsset.value
  ) {
    return;
  }
  selectedAsset.value.definition = editingAsset.value;
  assetStore.SAVE_ASSET(selectedAsset.value);
};
</script>
