<template>
  <RequestStatusMessage
    v-if="loadFuncDetailsReq.isPending && !editingFunc"
    :loadingMessage="`Loading function &quot;${selectedFuncSummary?.name}&quot;`"
    :requestStatus="loadFuncDetailsReq"
  />
  <ScrollArea
    v-else-if="
      selectedAsset &&
      loadFuncDetailsReq.isSuccess &&
      typeof editingFunc === 'string'
    "
    class="flex flex-col h-full border border-t-0 border-neutral-300 dark:border-neutral-600"
  >
    <template #top>
      <AssetEditorHeader
        :selectedAsset="selectedAsset"
        :selectedFunc="selectedFuncSummary"
      />
    </template>

    <CodeEditor
      :id="
        changeSetsStore.headChangeSetId !==
          changeSetsStore.selectedChangeSetId && selectedFuncCode
          ? `func-${selectedFuncCode.funcId}`
          : undefined
      "
      v-model="editingFunc"
      :typescript="selectedFuncCode?.types"
      @change="updateFuncCode"
      @close="emit('close')"
    />
  </ScrollArea>
  <ErrorMessage
    v-else-if="loadFuncDetailsReq.isError"
    :requestStatus="loadFuncDetailsReq"
  />
  <LoadingMessage v-else />
</template>

<script lang="ts" setup>
import { PropType, computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import {
  LoadingMessage,
  ErrorMessage,
  RequestStatusMessage,
  ScrollArea,
} from "@si/vue-lib/design-system";
import { FuncId } from "@/api/sdf/dal/func";
import { useFuncStore } from "@/store/func/funcs.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import CodeEditor from "@/components/CodeEditor.vue";
import { useAssetStore } from "@/store/asset.store";
import AssetEditorHeader from "../AssetEditorHeader.vue";

const changeSetsStore = useChangeSetsStore();
const assetStore = useAssetStore();

const props = defineProps({
  funcId: { type: String as PropType<FuncId>, required: true },
});

const funcStore = useFuncStore();
const { selectedFuncSummary, selectedFuncCode } = storeToRefs(funcStore);

const editingFunc = ref<string>(selectedFuncCode.value?.code ?? "");

const selectedAsset = computed(() => assetStore.selectedSchemaVariant);

const loadFuncDetailsReq = funcStore.getRequestStatus(
  "FETCH_CODE",
  props.funcId,
);

watch(
  selectedFuncCode,
  () => {
    if (!selectedFuncCode.value) {
      return;
    }

    // We have to ensure the changed func is the one we're looking at here, otherwise
    // we will copy the code from each the currently edited func into every func we've edited in
    // the past!
    if (
      selectedFuncCode.value.funcId === props.funcId &&
      editingFunc.value !== selectedFuncCode.value.code
    ) {
      editingFunc.value = selectedFuncCode.value.code;
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
const updateFuncCode = (code: string) => {
  if (updatedHead.value) return;

  updatedHead.value =
    changeSetsStore.selectedChangeSetId === changeSetsStore.headChangeSetId;
  funcStore.updateFuncCode(props.funcId, code);
};

const emit = defineEmits<{
  (e: "close"): void;
}>();
</script>
