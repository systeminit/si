<template>
  <div class="flex items-center mt-2">
    <div
      class="w-40 px-2 text-sm leading-tight text-right text-white input-label"
    >
      {{ registryProperty.name }}
    </div>

    <div
      v-if="!editMode"
      class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
      v-bind:class="textClasses"
    >
      {{ currentValue }}
    </div>
    <div
      class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
      v-else-if="editMode"
    >
      <select
        class="w-4/5 pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
        :data-cy="
          `editor-property-viewer-prop-${entityProperty.path.join('-')}`
        "
        :aria-label="entityProperty.name"
        v-bind:class="inputClasses"
        v-model="currentValue"
        @focus="storeStartingValue"
        @blur="saveIfModified"
      >
        <option
          v-for="option in entityProperty.prop.variants"
          v-bind:key="option"
        >
          {{ option }}
        </option>
      </select>
      <ValidationWidget
        :value="currentValue"
        :entityProperty="entityProperty"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState, mapGetters } from "vuex";
import _ from "lodash";

import PropMixin from "./PropMixin";
import ValidationWidget from "@/atoms/ValidationWidget.vue";

export default PropMixin.extend({
  name: "PropEnum",
  components: {
    ValidationWidget,
  },
});
</script>
