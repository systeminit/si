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
        <div class="flex flex-row items-center gap-xs pb-sm">
          <NodeSkeleton :color="selectedAsset.color" />
          <TruncateWithTooltip class="text-3xl font-bold">
            {{ schemaVariantDisplayName(selectedAsset) }}
          </TruncateWithTooltip>
        </div>
        <div class="text-sm italic flex flex-row flex-wrap gap-x-lg">
          <div>
            <span class="font-bold">Created At: </span>
            <Timestamp :date="selectedAsset.created_at" size="long" />
          </div>
          <!-- TODO: Populate the created by from SDF actorHistory-->
          <div>
            <span class="font-bold">Created By: </span>System Initiative
          </div>
          <SiChip v-if="isReadOnly" tone="warning" text="read-only" />
        </div>
      </div>
    </template>

    <CodeEditor
      :id="
        changeSetsStore.headChangeSetId === changeSetsStore.selectedChangeSetId
          ? undefined
          : `asset-${assetId}`
      "
      v-model="editingAsset"
      :disabled="isReadOnly"
      @change="onChange"
    />
    <!--
      TODO: jobelenus
      :typescript="selectedAsset?.types"
    -->
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
import { useAssetStore, schemaVariantDisplayName } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import SiChip from "@/components/SiChip.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import CodeEditor from "./CodeEditor.vue";
import NodeSkeleton from "./NodeSkeleton.vue";
import TruncateWithTooltip from "./TruncateWithTooltip.vue";

const changeSetsStore = useChangeSetsStore();

const props = defineProps<{
  assetId?: string;
}>();

const assetStore = useAssetStore();
const funcStore = useFuncStore();
const selectedAsset = computed(() =>
  props.assetId ? assetStore.variantsById[props.assetId] : undefined,
);

const selectedAssetFuncCode = computed(() => {
  const fId = selectedAsset.value?.assetFuncId;
  if (!fId) return null;
  return funcStore.funcDetailsById[fId]?.code;
});

const isReadOnly = computed(() => {
  return false;
});

const editingAsset = ref<string>(selectedAssetFuncCode.value ?? "");

const loadAssetReqStatus = assetStore.getRequestStatus(
  "LOAD_SCHEMA_VARIANT",
  props.assetId,
);

watch(
  () => selectedAsset.value,
  async () => {
    if (editingAsset.value !== selectedAssetFuncCode.value) {
      editingAsset.value = selectedAssetFuncCode.value ?? "";
    }
  },
  { immediate: true },
);

const updatedHead = ref(false);
watch(
  () => changeSetsStore.selectedChangeSetId,
  () => {
    updatedHead.value = false;
  },
);

const onChange = () => {
  if (
    !selectedAsset.value ||
    selectedAssetFuncCode.value === editingAsset.value ||
    updatedHead.value
  ) {
    return;
  }
  updatedHead.value =
    changeSetsStore.selectedChangeSetId === changeSetsStore.headChangeSetId;
  assetStore.enqueueVariantSave(
    {
      ...selectedAsset.value,
    },
    editingAsset.value,
  );
};
</script>
