<template>
  <RequestStatusMessage
    v-if="loadFuncDetailsReq.isPending && !editingFunc"
    :loadingMessage="`Loading function &quot;${selectedFuncSummary?.name}&quot;`"
    :requestStatus="loadFuncDetailsReq"
  />
  <ErrorMessage v-else-if="loadFuncDetailsReq.isError" :requestStatus="loadFuncDetailsReq" />
  <ScrollArea
    v-else-if="selectedAsset && selectedFuncCode && typeof editingFunc === 'string'"
    class="flex flex-col h-full border border-t-0 border-neutral-300 dark:border-neutral-600"
  >
    <template #top>
      <AssetEditorHeader :selectedAsset="selectedAsset" :selectedFunc="selectedFuncSummary" />
    </template>

    <CodeEditor
      :id="`func-${selectedFuncCode.funcId}`"
      ref="codeEditorRef"
      v-model="editingFunc"
      :disabled="selectedFuncSummary?.isLocked"
      :recordId="selectedFuncSummary?.funcId || ''"
      :typescript="selectedFuncSummary?.types || ''"
      @change="updateFuncCode"
      @close="emit('close')"
    />
  </ScrollArea>
  <LoadingMessage v-else />
</template>

<script lang="ts" setup>
import { PropType, computed, ref, watch, ComputedRef } from "vue";
import { ApiRequestStatus } from "@si/vue-lib/pinia";
import { LoadingMessage, ErrorMessage, RequestStatusMessage, ScrollArea } from "@si/vue-lib/design-system";
import { FuncId } from "@/api/sdf/dal/func";
import { useFuncStore } from "@/store/func/funcs.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import CodeEditor from "@/components/CodeEditor.vue";
import { useAssetStore } from "@/store/asset.store";
import AssetEditorHeader from "../AssetEditorHeader.vue";

const codeEditorRef = ref<InstanceType<typeof CodeEditor>>();

const changeSetsStore = useChangeSetsStore();
const assetStore = useAssetStore();

const props = defineProps({
  funcId: { type: String as PropType<FuncId>, required: true },
});

const funcStore = useFuncStore();
const selectedFuncSummary = computed(() => funcStore.selectedFuncSummary);
const selectedFuncCode = computed(() => funcStore.selectedFuncCode);

// note this is a space on purpose, CodeEditor has a fit with a fully empty string
const editingFunc = ref<string>(selectedFuncCode.value?.code ?? " ");

const selectedAsset = computed(() => assetStore.selectedSchemaVariant);

let loadFuncDetailsReq: ComputedRef<ApiRequestStatus>;

// changing props does not re-run this setup code
// instantiating `loadFuncDetailsReq` with `props.funcId`
// means that as props change, the request watcher *does not update*
// instead, use a watcher to re-assign the value each time
// this ensures that we get a new loading state when user selects another function
// on slower connections we were seeing old code in the editor, after a new code
// was selected, until it finally loaded and it all snapped into place
watch(
  () => props.funcId,
  () => {
    loadFuncDetailsReq = funcStore.getRequestStatus("FETCH_CODE", props.funcId);
  },
  { immediate: true },
);

watch(
  () => [selectedFuncCode.value?.funcId, selectedFuncCode.value?.code],
  () => {
    if (!selectedFuncCode.value) {
      return;
    }

    // We have to ensure the changed func is the one we're looking at here, otherwise
    // we will copy the code from each the currently edited func into every func we've edited in
    // the past! Also, we need to force an update to the content so that multiplayer edits will work.
    if (
      selectedFuncCode.value.funcId === props.funcId &&
      editingFunc.value.trimEnd() !== selectedFuncCode.value.code.trimEnd()
    ) {
      editingFunc.value = selectedFuncCode.value.code;
      codeEditorRef.value?.forceUpdateContent(selectedFuncCode.value.code);
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
const updateFuncCode = (funcId: string, code: string, debounce: boolean) => {
  if (updatedHead.value) return;
  if (!funcId) return; // protecting empty string, should never happen
  if (selectedFuncSummary.value?.isLocked) return;
  if (selectedFuncSummary.value?.funcId !== funcId) return;
  updatedHead.value = changeSetsStore.selectedChangeSetId === changeSetsStore.headChangeSetId;
  funcStore.updateFuncCode(funcId, code, debounce);
};

const emit = defineEmits<{
  (e: "close"): void;
}>();
</script>
