<template>
  <div>
    <RequestStatusMessage
      v-if="loadFuncDetailsReq.isPending"
      :request-status="loadFuncDetailsReq"
      :loading-message="`Loading function ${funcId}`"
    />
    <template v-else-if="loadFuncDetailsReq.isSuccess && editingFunc">
      <CodeEditor
        v-model="editingFunc"
        typescript
        :disabled="!isDevMode && isBuiltin"
        @change="updateFuncCode"
      />
    </template>
  </div>
</template>

<script lang="ts" setup>
import { PropType, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { FuncId, useFuncStore } from "@/store/func/funcs.store";
import CodeEditor from "@/components/CodeEditor.vue";
import RequestStatusMessage from "@/ui-lib/RequestStatusMessage.vue";

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
  async (selectedFunc) => {
    if (editingFunc.value !== selectedFuncDetails.value?.code) {
      editingFunc.value = selectedFuncDetails.value?.code ?? "";
    }
  },
  { immediate: true },
);

const updateFuncCode = (code: string) => {
  // funcStore.updateFuncCode(selectedFunc.value.id, code);
};
</script>
