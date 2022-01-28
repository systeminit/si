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

    <div v-if="output" class="flex flex-col flex-grow border-t border-gray-800">
      <div class="mt-2 mb-1 ml-2 text-xs font-medium output-title">Output</div>
      <div
        class="px-6 text-xs leading-relaxed whitespace-pre-line select-text output-lines"
      >
        {{ output }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import VueFeather from "vue-feather";
import { QualificationResult } from "@/api/sdf/dal/qualification";
import { computed, toRefs } from "vue";

const props = defineProps<{
  result: QualificationResult;
}>();

const { result } = toRefs(props);

const subChecks = computed(() => {
  if (result.value.sub_checks) {
    return result.value.sub_checks;
  } else {
    if (result.value.success) {
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
  }
});

const output = computed(() => {
  if (result.value.output) {
    return result.value.output.join("\n");
  } else {
    return "remove this mock output when we have real output!!!";
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
