<template>
  <RequestStatusMessage
    v-if="!loadAssetReqStatus || loadAssetReqStatus.isPending"
    :requestStatus="loadAssetReqStatus"
  />
  <div v-else-if="loadAssetReqStatus.isError" class="p-2 text-center text-neutral-400 dark:text-neutral-300">
    <template v-if="schemaVariantId"> Asset "{{ schemaVariantId }}" Function does not exist! </template>
    <template v-else>Select an asset to view it.</template>
  </div>
  <ScrollArea
    v-else-if="selectedAsset && editingAsset"
    class="flex flex-col h-full border border-t-0 border-neutral-300 dark:border-neutral-600"
  >
    <template #top>
      <AssetEditorHeader :selectedAsset="selectedAsset" />
    </template>

    <CodeEditor
      :id="
        changeSetsStore.headChangeSetId === changeSetsStore.selectedChangeSetId ? undefined : `asset-${schemaVariantId}`
      "
      v-model="editingAsset"
      :disabled="selectedAsset.isLocked"
      :recordId="selectedAsset.schemaVariantId"
      :typescript="editorTs || ''"
      @change="onChange"
    />
  </ScrollArea>
  <LoadingMessage v-else />
</template>

<script lang="ts" setup>
import { ref, watch, computed, onMounted, ComputedRef } from "vue";
import { ApiRequestStatus } from "@si/vue-lib/pinia";
import * as _ from "lodash-es";
import { RequestStatusMessage, ScrollArea, LoadingMessage } from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { editor_ts, loadEditorTs } from "@/utils/load_editor_ts";
import { SchemaVariantId } from "@/api/sdf/dal/schema";
import CodeEditor from "./CodeEditor.vue";
import AssetEditorHeader from "./AssetEditorHeader.vue";

const changeSetsStore = useChangeSetsStore();
const editorTs = ref<string | null>(null);

const props = defineProps<{
  schemaVariantId?: SchemaVariantId;
}>();

const assetStore = useAssetStore();
const funcStore = useFuncStore();
const selectedAsset = computed(() =>
  props.schemaVariantId ? assetStore.variantFromListById[props.schemaVariantId] : undefined,
);

const selectedAssetFuncCode = computed(() => {
  const fId = selectedAsset.value?.assetFuncId;
  if (!fId) return null;
  return funcStore.funcCodeById[fId]?.code;
});

const editingAsset = ref<string>(selectedAssetFuncCode.value ?? "");

let loadAssetReqStatus: ComputedRef<ApiRequestStatus>;

watch(
  () => selectedAsset.value,
  () => {
    loadAssetReqStatus = funcStore.getRequestStatus("FETCH_CODE", selectedAsset.value?.assetFuncId);
  },
  { immediate: true },
);

watch(
  [() => editingAsset.value, () => selectedAssetFuncCode.value],
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

const onChange = (_schemaVariantId: string, code: string, debounce: boolean) => {
  if (
    !selectedAsset.value ||
    selectedAsset.value.isLocked ||
    selectedAssetFuncCode.value === editingAsset.value ||
    updatedHead.value
  ) {
    return;
  }
  updatedHead.value = changeSetsStore.selectedChangeSetId === changeSetsStore.headChangeSetId;
  if (!updatedHead.value) {
    const asset = _.cloneDeep(selectedAsset.value);
    assetStore.enqueueVariantSave(
      {
        ...asset,
      },
      code,
      debounce,
    );
  }
};

onMounted(async () => {
  if (!editor_ts) {
    editorTs.value = await loadEditorTs();
  } else {
    editorTs.value = editor_ts;
  }
});
</script>
