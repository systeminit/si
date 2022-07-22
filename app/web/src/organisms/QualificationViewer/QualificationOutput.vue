<template>
  <div class="w-full pb-1">
    <div class="pt-3 pb-4 border-t border-gray-800">
      <div
        v-for="(subCheck, index) in subChecks"
        :key="index"
        class="flex flex-col flex-grow"
      >
        <div class="flex flex-row items-center px-6 pt-1 text-xs output-lines">
          <div class="w-4">
            <VueFeather type="check" size="1.1em" class="mr-2 text-xs" />
          </div>
          <div class="flex flex-row">
            <div class="ml-1">{{ subCheck.description }}</div>

            <div class="ml-1 text-xs">(</div>
            <span v-if="subCheck.status === 'Success'" class="success">
              success
            </span>
            <span v-else-if="subCheck.status === 'Failure'" class="error">
              failure
            </span>
            <span v-else class="unknown">unknown</span>
            <div class="">)</div>
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="combinedOutput"
      class="flex flex-col flex-grow border-t border-gray-800"
    >
      <div class="mt-2 mb-1 ml-2 text-xs font-medium output-title">Output</div>
      <div
        class="px-6 text-xs leading-relaxed whitespace-pre-line select-text output-lines"
      >
        {{ combinedOutput }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import VueFeather from "vue-feather";
import {
  QualificationOutputStream,
  QualificationResult,
} from "@/api/sdf/dal/qualification";
import { computed } from "vue";

const props = defineProps<{
  result: QualificationResult;
  output?: Array<QualificationOutputStream>;
}>();

const subChecks = computed(() => {
  if (props.result.sub_checks.length > 0) {
    return props.result.sub_checks;
  } else if (props.result.success) {
    return [
      {
        status: "Success",
        description: "Qualification check succeeded",
      },
    ];
  } else {
    return [
      {
        status: "Failure",
        description: "Qualification check failed",
      },
    ];
  }
});

const combinedOutput = computed((): string => {
  if (props.output) {
    const combined = props.output.map((entry) => entry.line).join("\n");
    return combined;
  } else {
    return "";
  }
});
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
