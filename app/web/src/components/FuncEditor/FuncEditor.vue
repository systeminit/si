<template>
  <CodeEditor
    v-model="editingFunc"
    typescript
    :disabled="!isDevMode && isBuiltin"
    @change="updateFuncCode"
  />
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useFuncStore } from "@/store/func/funcs.store";
import CodeEditor from "@/components/CodeEditor.vue";

const funcStore = useFuncStore();
const { selectedFunc } = storeToRefs(funcStore);

const isDevMode = import.meta.env.DEV;

const editingFunc = ref<string>(selectedFunc.value?.code ?? "");
const isBuiltin = ref<boolean>(selectedFunc.value?.isBuiltin ?? false);

watch(
  () => selectedFunc.value,
  async (selectedFunc) => {
    if (editingFunc.value !== selectedFunc.code) {
      editingFunc.value = selectedFunc.code ?? "";
    }

    isBuiltin.value = selectedFunc.isBuiltin;
  },
  { immediate: true },
);

const updateFuncCode = (code: string) => {
  funcStore.updateFuncCode(selectedFunc.value.id, code);
};
</script>
