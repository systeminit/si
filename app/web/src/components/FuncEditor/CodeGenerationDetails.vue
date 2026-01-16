<template>
  <div class="p-xs flex flex-col gap-xs">
    <LeafInputs v-model="inputs" :kind="FuncBindingKind.CodeGeneration" :disabled="func?.isLocked" @change="update" />
  </div>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import * as _ from "lodash-es";
import { useFuncStore } from "@/store/func/funcs.store";
import { FuncBindingKind } from "@/api/sdf/dal/func";
import LeafInputs from "./LeafInputs.vue";

const funcStore = useFuncStore();

const props = defineProps<{
  schemaVariantId: string;
  funcId: string;
}>();

const func = computed(() => {
  return funcStore.funcsById[props.funcId];
});

const binding = computed(() => {
  const bindings = funcStore.codegenBindings[props.funcId];
  const binding = bindings?.filter((b) => b.schemaVariantId === props.schemaVariantId).pop();
  return binding;
});

const inputs = ref(_.clone(binding.value?.inputs) || []);

const update = () => {
  if (binding.value) {
    binding.value.inputs = _.clone(inputs.value);
    funcStore.UPDATE_BINDING(props.funcId, [binding.value]);
  }
};

const detachFunc = () => {
  if (binding.value) funcStore.DELETE_BINDING(props.funcId, [binding.value]);
};

defineExpose({ detachFunc });
</script>
