<template>
  <div class="w-full mb-4">
    <div v-if="parseKubevalOutput">
      <div
        v-for="o in parseKubevalOutput"
        :key="o.filename"
        class="flex flex-col flex-grow border border-gray-800"
      >
        <div class="px-6 py-1 text-xs bg-gray-800">
          {{ o.kind }} ({{ o.status }})
        </div>

        <div
          v-for="e in o.errors"
          :key="e"
          class="flex flex-col px-1 my-1 text-xs select-text pl-7 output-lines "
        >
          <div class="flex flex-row items-center">
            <AlertOctagonIcon size="1x" class="error" />
            <div class="ml-2">
              {{ e }}
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-else-if="parseValidFieldsOutput">
      <div
        v-for="(validation, fieldName) in parseValidFieldsOutput"
        :key="fieldName"
        class="flex flex-col flex-grow border border-gray-800"
      >
        <div class="px-6 py-1 text-xs bg-gray-800">
          field {{ fieldName }}
          <span v-if="validation == 'success'">(success)</span>
          <span v-else>(failure)</span>
        </div>

        <template v-if="validation && validation != 'success'">
          <div
            v-for="e in validation"
            :key="e.message"
            class="flex flex-col px-1 my-1 text-xs select-text pl-7 output-lines "
          >
            <div class="flex flex-row items-center">
              <AlertOctagonIcon size="1x" class="error" />
              <div class="ml-2">
                {{ e.message }}
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>

    <div v-else class="flex flex-col flex-grow border border-gray-800">
      <div
        class="px-6 py-4 text-xs leading-relaxed whitespace-pre-line bg-gray-900 select-text output-lines "
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
  AlertOctagonIcon,
} from "vue-feather-icons";
// import CodeMirror from "@/molecules/CodeMirror.vue";

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
    AlertOctagonIcon,
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
.error {
  color: #ff8f8f;
}

.output-lines {
  color: #cfc1bb;
}
</style>
