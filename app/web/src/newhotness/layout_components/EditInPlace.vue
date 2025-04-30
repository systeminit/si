<template>
  <!-- eslint-disable vue/no-multiple-template-root -->
  <slot v-if="!editing" name="trigger" />
  <slot v-else name="input" />
</template>

<script lang="ts" setup>
import { ref } from "vue";

const editing = ref(false);

const toggle = () => {
  editing.value = !editing.value;
  if (editing.value) emit("showing");
  else emit("hidden");
};
const hide = () => {
  editing.value = false;
  // no emit on purpose
};
defineExpose({ toggle, hide });
const emit = defineEmits<{
  (e: "hidden"): void;
  (e: "showing"): void;
}>();
</script>
