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
      <CodeEditor v-model="editingFunc" typescript @change="updateFuncCode" />
    </template>
    <ErrorMessage
      v-else-if="loadFuncDetailsReq.isError"
      :request-status="loadFuncDetailsReq"
    />
    <LoadingMessage v-else no-message />
  </div>
</template>

<script lang="ts" setup>
import { PropType, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { LoadingMessage, ErrorMessage } from "@si/vue-lib/design-system";
import { FuncId, useFuncStore } from "@/store/func/funcs.store";
import CodeEditor from "@/components/CodeEditor.vue";

const props = defineProps({
  funcId: { type: String as PropType<FuncId>, required: true },
});

const funcStore = useFuncStore();
const { selectedFuncSummary, selectedFuncDetails } = storeToRefs(funcStore);

const editingFunc = ref<string>(selectedFuncDetails.value?.code ?? "");

const loadFuncDetailsReq = funcStore.getRequestStatus(
  "FETCH_FUNC_DETAILS",
  props.funcId,
);

watch(
  selectedFuncDetails,
  () => {
    if (!selectedFuncDetails.value) {
      return;
    }

    // We have to ensure the changed func is the one we're looking at here, otherwise
    // we will copy the code from each the currently edited func into every func we've edited in
    // the past!
    if (
      selectedFuncDetails.value.id === props.funcId &&
      editingFunc.value !== selectedFuncDetails.value.code
    ) {
      editingFunc.value = selectedFuncDetails.value.code;
    }
  },
  { immediate: true },
);

const updateFuncCode = (code: string) => {
  funcStore.updateFuncCode(props.funcId, code);
};
</script>
