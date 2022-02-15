<template>
  <button
    v-if="(props.editValue ?? undefined) !== undefined"
    @click="props.unset && props.unset()"
  >
    <VueFeather type="trash-2" size="1em" class="ml-1" />
  </button>
</template>

<script setup lang="ts">
import type { EditFieldValue } from "@/api/sdf/dal/edit_field";
import VueFeather from "vue-feather";

const props = defineProps<{
  editValue?: EditFieldValue;
  unset: () => void;
}>();
</script>

<style scoped>
/* Is this what we want? It's how the demo works */
button:focus {
  outline: 1px dotted;
}
</style>
