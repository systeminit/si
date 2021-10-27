<template>
  <div class="flex flex-row items-center w-full" v-if="showField">
    <div class="flex flex-col w-full">
      <div class="flex flex-row items-center">
        <div
          class="flex-wrap self-start text-sm leading-tight text-right w-36"
          :class="nameClasses"
          v-if="name"
        >
          {{ name }}
        </div>
        <div class="flex w-full">
          <div
            class="flex mx-2 text-sm leading-tight text-gray-400"
            v-if="editMode"
            @keyup.stop
            @keydown.stop
          >
            <!-- could flex-grow if needed -->
            <slot name="widget" />
          </div>

          <div v-else class="flex mx-2 text-sm leading-tight text-gray-400">
            <!-- could flex-grow if needed -->
            <slot name="value" />
          </div>
        </div>
      </div>

      <div class="flex flex-wrap">
        <ValidationErrors :errors="errors" class="p-2" />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import { ValidateFailure } from "si-entity/dist/validation";
import ValidationErrors from "@/organisims/AttributeViewer/ValidationErrors.vue";

export default Vue.extend({
  name: "Field",
  components: {
    ValidationErrors,
  },
  props: {
    name: {
      type: String,
    },
    showField: {
      type: Boolean,
      required: true,
    },
    errors: {
      type: Array as PropType<ValidateFailure["errors"]>,
      required: true,
    },
    editMode: {
      type: Boolean,
      required: true,
    },
    nameClasses: {
      type: Object as PropType<Record<string, boolean>>,
    },
  },
});
</script>
