<template>
  <div class="flex flex-row items-center w-full mt-2" v-if="showField">
    <div class="flex flex-col w-full">
      <div class="flex flex-row items-center w-full">
        <div
          class="w-1/4 break-all px-2 text-sm leading-tight text-right flex-wrap"
          :class="nameClasses"
          v-if="name"
        >
          {{ name }}
        </div>
        <div
          class="flex flex-grow pl-2 mr-2 mr-10 text-sm leading-tight text-gray-400"
          v-if="editMode"
          @keyup.stop
          @keydown.stop
        >
          <slot name="widget" />
        </div>
        <div
          v-else
          class="flex flex-grow pl-2 mr-2 text-sm leading-tight text-gray-400"
        >
          <slot name="value" />
        </div>
      </div>
      <div class="flex flex-row w-full">
        <div class="w-40"></div>
        <div class="flex flex-grow pl-2 mr-10">
          <ValidationErrors :errors="errors" />
        </div>
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
