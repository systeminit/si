<template>
  <div class="w-full pb-1">
    <div v-if="parseKubevalOutput">
      <div class="text-xs">TODO(nick): implement parseKubevalOutput</div>
    </div>

    <div
      v-else-if="parseValidFieldsOutput"
      class="pt-3 pb-4 border-t border-gray-800"
    >
      <div class="text-xs">TODO(nick): implement parseValidFieldsOutput</div>
    </div>

    <div v-else class="flex flex-col flex-grow border-t border-gray-800">
      <div class="mt-2 mb-1 ml-2 text-xs font-medium output-title">Output</div>
      <div
        class="px-6 text-xs leading-relaxed whitespace-pre-line select-text output-lines"
      >
        {{ data }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { QualificationResult } from "@/api/sdf/dal/qualification";
import { computed } from "vue";

// TODO(nick): remove dummy values.
const parseKubevalOutput = false;
const parseValidFieldsOutput = false;

// TODO(nick): determine how to handle multiple errors since veritech only returns one error message (currently).
const data = computed(() => {
  let data = "";
  for (let error of props.result.errors) {
    if (data === "") {
      data = error.message;
    } else {
      data = data + "\n" + error.message;
    }
  }
  return data;
});

const props = defineProps<{
  result: QualificationResult;
}>();
</script>

<style scoped>
.success {
  color: #4bde80;
}

.error {
  color: #fb7185;
}

.unknown {
  color: #969696;
}

.output-lines {
  color: #e5decf;
}

.output-title {
  color: #e5e5e5;
}
</style>
