<template>
  <div class="w-full pb-1">
    <div v-if="showParseKubeValOutput">
      <div
        v-for="o in parseKubevalOutput"
        :key="o.filename"
        class="flex flex-col flex-grow"
      >
        <div
          class="flex flex-row items-center px-6 pt-3 pb-1 border-t border-gray-800"
        >
          <div class="w-4">CheckIcon</div>
          <div class="text-xs output-lines">Kind and Status</div>
        </div>
        <div
          v-for="e in o.errors"
          :key="e"
          class="flex flex-col pl-10 mb-1 ml-1 text-xs select-text output-lines"
        >
          <div class="flex flex-row items-center">
            AlertTriangleIcon
            <div class="ml-2 text-xs error">
              {{ e }}
            </div>
          </div>
        </div>
      </div>
    </div>
    <div
      v-else-if="parseValidFieldsOutput"
      class="pt-3 pb-4 border-t border-gray-800"
    >
      <div
        v-for="(validation, fieldName) in parseValidFieldsOutput"
        :key="fieldName"
        class="flex flex-col flex-grow"
      >
        <div class="flex flex-row items-center px-6 pt-1 text-xs output-lines">
          <div class="w-4">CheckIcon</div>
          <div class="flex flex-row">
            <div class="ml-1">field {{ fieldName }}</div>

            <div class="ml-1 text-xs">(</div>
            <span v-if="showSuccess" class="success">success</span>
            <span v-else class="error">failure</span>
            <div class="">)</div>
          </div>
        </div>

        <template v-if="validation && showSuccess">
          <div
            v-for="e in validation"
            :key="e.message"
            class="flex flex-col pl-10 mt-1 mb-1 ml-2 text-xs select-text output-lines"
          >
            <div class="flex flex-row items-center">
              AlertTriangleIcon
              <div class="ml-2 error">
                {{ e.message }}
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>

    <div v-else class="flex flex-col flex-grow border-t border-gray-800">
      <div class="mt-2 mb-1 ml-2 text-xs font-medium output-title">Output</div>
      <div
        class="px-6 text-xs leading-relaxed whitespace-pre-line select-text output-lines"
      >
        Data
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const parseKubevalOutput = [
  { filename: "foo", errors: [{ message: "poop" }] },
  { filename: "bar", errors: [{ message: "canoe" }] },
];
const parseValidFieldsOutput = [
  { validation: "success", fieldName: "i like" },
  { validation: "failure", fieldName: "my butt" },
];
const showParseKubeValOutput = true;
const showSuccess = true;
</script>
