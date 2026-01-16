<template>
  <div class="p-3 flex flex-col gap-xs">
    <LeafInputs v-model="inputs" :disabled="func?.isLocked" :kind="FuncBindingKind.Qualification" @change="update" />
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
  funcId: string;
  schemaVariantId: string;
  disabled?: boolean;
}>();

const func = computed(() => {
  return funcStore.funcsById[props.funcId];
});

const binding = computed(() => {
  const bindings = funcStore.qualificationBindings[props.funcId];
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
