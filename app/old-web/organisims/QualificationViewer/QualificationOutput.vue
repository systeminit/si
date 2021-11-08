<template>
  <div class="w-full pb-1">
    <div v-if="parseKubevalOutput">
      <div
        v-for="o in parseKubevalOutput"
        :key="o.filename"
        class="flex flex-col flex-grow"
      >
        <div
          class="flex flex-row items-center px-6 pt-3 pb-1 border-t border-gray-800 "
        >
          <div class="w-4">
            <CheckIcon
              size="1x"
              class="mr-2 text-xs"
              v-if="o.errors.length == 0"
            />
          </div>
          <div class="text-xs output-lines">{{ o.kind }} ({{ o.status }})</div>
        </div>
        <div
          v-for="e in o.errors"
          :key="e"
          class="flex flex-col pl-10 mb-1 ml-1 text-xs select-text output-lines"
        >
          <div class="flex flex-row items-center">
            <AlertTriangleIcon size="1x" class="error" />
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
          <div class="w-4">
            <CheckIcon
              size="1x"
              class="mr-2 text-xs"
              v-if="validation == 'success'"
            />
          </div>
          <div class="flex flex-row">
            <div class="ml-1">field {{ fieldName }}</div>

            <div class="ml-1 text-xs">(</div>
            <span v-if="validation == 'success'" class="success">success</span>
            <span v-else class="error">failure</span>
            <div class="">)</div>
          </div>
        </div>

        <template v-if="validation && validation != 'success'">
          <div
            v-for="e in validation"
            :key="e.message"
            class="flex flex-col pl-10 mt-1 mb-1 ml-2 text-xs select-text output-lines"
          >
            <div class="flex flex-row items-center">
              <AlertTriangleIcon size="1x" class="error" />
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
        class="px-6 text-xs leading-relaxed whitespace-pre-line select-text output-lines "
      >
        {{ data }}
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import {
  // CheckSquareIcon,
  AlertTriangleIcon,
  CheckIcon,
} from "vue-feather-icons";

// [
//     {
//         "filename": "fixtures/invalid.yaml",
//         "kind": "ReplicationController",
//         "status": "invalid",
//         "errors": [
//                 "spec.replicas: Invalid type. Expected: [integer,null], given: string"
//         ]
//     }
// ]

interface KubevalOutput {
  filename: string;
  kind: string;
  status: string;
  errors: string[];
}

interface ValidFieldsOutput {
  [fieldPath: string]: "success" | { message: string }[];
}

export default Vue.extend({
  name: "QualificationOutput",
  components: {
    AlertTriangleIcon,
    CheckIcon,
  },
  props: {
    kind: {
      type: String,
      required: true,
    },
    data: {
      type: String,
      required: true,
    },
  },
  computed: {
    parseKubevalOutput(): KubevalOutput | null {
      if (this.kind == "kubeval") {
        return JSON.parse(this.data) as KubevalOutput;
      } else {
        return null;
      }
    },
    parseValidFieldsOutput(): ValidFieldsOutput | null {
      if (this.kind == "allFieldsValid") {
        return JSON.parse(this.data) as ValidFieldsOutput;
      } else {
        return null;
      }
    },
  },
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
