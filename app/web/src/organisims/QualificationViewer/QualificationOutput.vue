<template>
  <div class="w-full pb-1">
    <div v-if="parseKubevalOutput">
      <div class="text-xs">Not yet implemented: parseKubevalOutput</div>
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
          <div class="w-4">
            <VueFeather
              v-if="validation === 'success'"
              type="check"
              size="1.1em"
              class="mr-2 text-xs"
            />
          </div>
          <div class="flex flex-row">
            <div class="ml-1">field {{ fieldName }}</div>

            <div class="ml-1 text-xs">(</div>
            <span v-if="validation === 'success'" class="success">success</span>
            <span v-else class="error">failure</span>
            <div class="">)</div>
          </div>
        </div>
        <template v-if="validation && validation !== 'success'">
          <div
            v-for="e in validation"
            :key="e.message"
            class="flex flex-col pl-10 mt-1 mb-1 ml-2 text-xs select-text output-lines"
          >
            <div class="flex flex-row items-center">
              <VueFeather type="alert-triangle" size="1.1em" class="error" />
              <div class="ml-2 error">
                {{ e.message }}
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- NOTE(nick): remove "v-else" directive for now. Always render data, even if empty.
    The only exception to this is the special "allFieldsValid" box.
    -->
    <div
      v-if="!allFieldsValid"
      class="flex flex-col flex-grow border-t border-gray-800"
    >
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
import VueFeather from "vue-feather";
import { computed } from "vue";

const props = defineProps<{
  kind: string;
  // NOTE(nick): data can come in as empty. This might be true if we only have a description to display, but not a
  // result. We should account for that scenario as we move past old-web parity.
  data: string;
  // TODO(nick): we need to display descriptions at some point. This was not in old-web.
  description?: string;
}>();

interface ValidFieldsOutput {
  [fieldPath: string]: "success" | { message: string }[];
}

const allFieldsValid = computed((): boolean => {
  return props.kind == "allFieldsValid";
});

// FIXME(nick): we hardcode this value for now to showcase the UI.
const parseValidFieldsOutput = computed((): ValidFieldsOutput | false => {
  if (props.data === "") {
    return false;
  }
  if (props.kind === "allFieldsValid") {
    return {
      success: "success",
    };
  }
  return {
    someFieldPath: [{ message: props.data }],
  };
});

// FIXME(nick): re-introduce this once we have the ability to concatenate validations.
// const parseValidFieldsOutput = computed(() => {
//   if (props.kind === "allFieldsValid") {
//     return JSON.parse(props.data) as ValidFieldsOutput;
//   }
//   return null;
// });

// TODO(nick): remove dummy values.
const parseKubevalOutput = false;
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
