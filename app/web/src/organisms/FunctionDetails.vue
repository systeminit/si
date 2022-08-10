<template>
  <div v-if="func.id > 0" class="text-center">
    <SiTextBox2
      id="handler"
      v-model="handler"
      class="w-full"
      title="Entrypoint"
    />
    <SiTextBox2 id="name" v-model="name" title="Name" class="w-full" />
  </div>
  <div v-else>Please select a function to see its properties.</div>
</template>

<script setup lang="ts">
import SiTextBox2 from "@/atoms/SiTextBox2.vue";
import { ListedFuncView } from "@/service/func/list_funcs";
import { ref, watch } from "vue";

const props = defineProps<{
  func: ListedFuncView;
}>();

const name = ref(props.func.name);
const handler = ref(props.func.handler);

watch(
  () => props.func,
  (func) => {
    name.value = func.name;
    handler.value = func.handler;
  },
);

const emit = defineEmits<{
  (e: "updatedName", v: string): void;
  (e: "updatedHandler", v: string): void;
}>();

const setName = (handler: string) => emit("updatedName", handler);
const setHandler = (name: string) => emit("updatedHandler", name);

watch(handler, (newValue) => setHandler(newValue ?? ""));
watch(name, (newValue) => setName(newValue));
</script>
