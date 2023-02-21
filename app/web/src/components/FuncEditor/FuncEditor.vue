<template>
  <div>
    <div
      v-if="loadFuncDetailsReq.isPending && !editingFunc"
      class="w-full flex flex-col items-center gap-4 p-xl"
    >
      <LoadingMessage
        >Loading function "{{ selectedFuncSummary?.name }}"</LoadingMessage
      >
    </div>
    <template v-else-if="loadFuncDetailsReq.isSuccess && editingFunc">
      <CodeEditor
        v-model="editingFunc"
        typescript
        :disabled="!isDevMode && isBuiltin"
        @change="updateFuncCode"
      />
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
import { FuncId, useFuncStore } from "@/store/func/funcs.store";
import CodeEditor from "@/components/CodeEditor.vue";
import LoadingMessage from "@/ui-lib/LoadingMessage.vue";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";

const props = defineProps({
  funcId: { type: String as PropType<FuncId>, required: true },
});

const funcStore = useFuncStore();
const { selectedFuncSummary, selectedFuncDetails } = storeToRefs(funcStore);

const isDevMode = import.meta.env.DEV;

const editingFunc = ref<string>(selectedFuncDetails.value?.code ?? "");
const isBuiltin = ref<boolean>(selectedFuncSummary.value?.isBuiltin ?? false);

const loadFuncDetailsReq = funcStore.getRequestStatus(
  "FETCH_FUNC_DETAILS",
  props.funcId,
);

watch(
  selectedFuncDetails,
  () => {
    if (editingFunc.value !== selectedFuncDetails.value?.code) {
      editingFunc.value = selectedFuncDetails.value?.code ?? "";
    }
  },
  { immediate: true },
);

const updateFuncCode = (code: string) => {
  funcStore.updateFuncCode(props.funcId, code);
};
</script>
