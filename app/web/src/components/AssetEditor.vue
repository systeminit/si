<template>
  <RequestStatusMessage
    v-if="loadAssetReqStatus.isPending"
    :requestStatus="loadAssetReqStatus"
  />
  <ScrollArea
    v-else-if="assetId && selectedAsset"
    class="flex flex-col h-full border border-t-0 border-neutral-300 dark:border-neutral-600"
  >
    <template #top>
      <div class="p-sm">
        <div class="flex flex-row items-center gap-2 pb-sm">
          <NodeSkeleton :color="`#${selectedAsset.color}`" />
          <div class="text-3xl font-bold truncate">
            {{ assetDisplayName(selectedAsset) }}
          </div>
        </div>
        <div class="text-sm italic flex flex-row flex-wrap gap-x-lg">
          <div>
            <span class="font-bold">Created At: </span>
            <Timestamp :date="selectedAsset.createdAt" size="long" />
          </div>
          <!-- TODO: Populate the created by from SDF actorHistory-->
          <div>
            <span class="font-bold">Created By: </span>System Initiative
          </div>
          <SiChip v-if="isReadOnly" variant="warning" text="read-only" />
        </div>
      </div>
    </template>

    <CodeEditor
      :id="`asset-${assetId}`"
      v-model="editingAsset"
      :typescript="selectedAsset?.types"
      :disabled="isReadOnly"
      @save="onChange"
    />
  </ScrollArea>
  <div v-else class="p-2 text-center text-neutral-400 dark:text-neutral-300">
    <template v-if="assetId">Asset "{{ assetId }}" does not exist!</template>
    <template v-else>Select an asset to view it.</template>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch, computed } from "vue";
import {
  Timestamp,
  RequestStatusMessage,
  ScrollArea,
} from "@si/vue-lib/design-system";
import { useAssetStore, assetDisplayName } from "@/store/asset.store";
import SiChip from "@/components/SiChip.vue";
import CodeEditor from "./CodeEditor.vue";
import NodeSkeleton from "./NodeSkeleton.vue";

const props = defineProps<{
  assetId?: string;
}>();

const assetStore = useAssetStore();
const selectedAsset = computed(() =>
  props.assetId ? assetStore.assetsById[props.assetId] : undefined,
);

const isReadOnly = computed(
  () =>
    !!(selectedAsset.value?.hasComponents || selectedAsset.value?.hasAttrFuncs),
);

const editingAsset = ref<string>(selectedAsset.value?.code ?? "");

const loadAssetReqStatus = assetStore.getRequestStatus(
  "LOAD_ASSET",
  props.assetId,
);

watch(
  () => selectedAsset.value,
  async (selectedAsset) => {
    if (editingAsset.value !== selectedAsset?.code) {
      editingAsset.value = selectedAsset?.code ?? "";
    }
  },
  { immediate: true },
);

const onChange = () => {
  if (!selectedAsset.value || selectedAsset.value.code === editingAsset.value) {
    return;
  }
  assetStore.enqueueAssetSave({
    ...selectedAsset.value,
    code: editingAsset.value,
  });
};
</script>
