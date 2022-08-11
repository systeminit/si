<template>
  <div v-if="funcId > 0" class="text-center h-full overflow-y-auto">
    <SiTextBox
      id="handler"
      v-model="editingFunc.modifiedFunc.handler"
      class="w-full"
      title="Entrypoint"
      @blur="updateFunc"
    />
    <SiTextBox
      id="name"
      v-model="editingFunc.modifiedFunc.name"
      title="Name"
      class="w-full"
      @blur="updateFunc"
    />
    <div
      class="absolute bottom-0 w-full h-12 text-right p-2 border-t border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800"
    >
      <SiButton icon="save" kind="save" label="Save Qualifications" size="lg" />
    </div>
  </div>
  <div v-else class="p-2 text-center text-neutral-400">
    Select a function to view its properties.
  </div>
</template>

<script setup lang="ts">
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiButton from "@/atoms/SiButton.vue";
import { toRef, computed } from "vue";
import { funcState, changeFunc, nullEditingFunc } from "./func_state";

const props = defineProps<{
  funcId: number;
}>();

const funcId = toRef(props, "funcId", -1);

const editingFunc = computed(
  () => funcState.funcs.find((f) => f.id == funcId.value) ?? nullEditingFunc,
);

const updateFunc = () => {
  changeFunc({ ...editingFunc.value.modifiedFunc });
};
</script>
