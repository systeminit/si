<template>
  <div
    class="h-full border border-t-0 border-neutral-300 dark:border-neutral-600"
  >
    <div
      v-if="loadFuncDetailsReq.isPending && !editingFunc"
      class="w-full flex flex-col items-center gap-4 p-xl"
    >
      <LoadingMessage
        >Loading function "{{ selectedFuncSummary?.name }}"</LoadingMessage
      >
    </div>
    <template
      v-else-if="
        loadFuncDetailsReq.isSuccess && typeof editingFunc === 'string'
      "
    >
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
    </template>
    <ErrorMessage
      v-else-if="loadFuncDetailsReq.isError"
      :requestStatus="loadFuncDetailsReq"
    />
    <LoadingMessage v-else />
  </div>
</template>

<script lang="ts" setup>
import { PropType, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { LoadingMessage, ErrorMessage } from "@si/vue-lib/design-system";
import { FuncId } from "@/api/sdf/dal/func";
import { useFuncStore } from "@/store/func/funcs.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import CodeEditor from "@/components/CodeEditor.vue";

const changeSetsStore = useChangeSetsStore();

const props = defineProps({
  funcId: { type: String as PropType<FuncId>, required: true },
});

const funcStore = useFuncStore();
const { selectedFuncSummary, selectedFuncCode } = storeToRefs(funcStore);

const editingFunc = ref<string>(selectedFuncCode.value?.code ?? "");

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
