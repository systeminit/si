<template>
  <div class="flex items-center mt-2" v-if="currentValue || editMode">
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
      class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400 justiy-start"
      v-else
    >
      <input
        class="pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
        :data-cy="
          `editor-property-viewer-prop-${registryProperty.path.join('-')}`
        "
        v-bind:class="inputClasses"
        type="checkbox"
        :aria-label="registryProperty.name"
        v-model="currentValue"
        @focus="storeStartingValue"
        @blur="saveIfModified"
      />
      <ValidationWidget
        :value="currentValue"
        :registryProperty="registryProperty"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { Store, mapState, mapGetters } from "vuex";
import _ from "lodash";

import PropMixin from "./PropMixin";
import ValidationWidget from "@/atoms/ValidationWidget.vue";

export default PropMixin.extend({
  name: "PropBool",
  components: {
    ValidationWidget,
  },
});
</script>
