<template>
  <RequestStatusMessage
    v-if="loadAssetReqStatus.isPending"
    :requestStatus="loadAssetReqStatus"
  />
  <ScrollArea
    v-else-if="selectedAsset"
    class="flex flex-col h-full border border-t-0 border-neutral-300 dark:border-neutral-600"
  >
    <template #top>
      <AssetEditorHeader :selectedAsset="selectedAsset" />
    </template>

    <CodeEditor
      :id="
        changeSetsStore.headChangeSetId === changeSetsStore.selectedChangeSetId
          ? undefined
          : `asset-${assetId}`
      "
      v-model="editingAsset"
      :disabled="selectedAsset.isLocked"
      :recordId="selectedAsset.schemaVariantId"
      :typescript="editorTs || ''"
      @change="onChange"
    />
  </ScrollArea>
  <div
    v-else-if="loadAssetReqStatus.isError"
    class="p-2 text-center text-neutral-400 dark:text-neutral-300"
  >
    <template v-if="assetId">Asset "{{ assetId }}" does not exist!</template>
    <template v-else>Select an asset to view it.</template>
  </div>
  <LoadingMessage v-else />
</template>

<script lang="ts" setup>
import { ref, watch, computed, onMounted } from "vue";
import {
  RequestStatusMessage,
  ScrollArea,
  LoadingMessage,
} from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { editor_ts, loadEditorTs } from "@/utils/load_editor_ts";
import CodeEditor from "./CodeEditor.vue";
import AssetEditorHeader from "./AssetEditorHeader.vue";

const changeSetsStore = useChangeSetsStore();
const editorTs = ref<string | null>(null);

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
  return funcStore.funcCodeById[fId]?.code;
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

const onChange = (_: string, code: string) => {
  if (
    !selectedAsset.value ||
    selectedAssetFuncCode.value === editingAsset.value ||
    updatedHead.value
  ) {
    return;
  }
  updatedHead.value =
    changeSetsStore.selectedChangeSetId === changeSetsStore.headChangeSetId;
  if (!updatedHead.value)
    assetStore.enqueueVariantSave(
      {
        ...selectedAsset.value,
      },
      code,
    );
};

onMounted(async () => {
  if (!editor_ts) {
    editorTs.value = await loadEditorTs();
  } else {
    editorTs.value = editor_ts;
  }
});
</script>
