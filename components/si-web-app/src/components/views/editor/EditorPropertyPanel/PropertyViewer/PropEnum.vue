<template>
  <div class="flex items-center mt-2">
    <div
      class="w-40 px-2 text-sm leading-tight text-right text-white input-label"
    >
      {{ entityProperty.name }}
    </div>

    <div
      v-if="editorMode == 'view'"
      class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
      v-bind:class="textClasses"
    >
      {{ fieldValue }}
    </div>
    <div
      class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
      v-else-if="editorMode == 'edit'"
    >
      <select
        class="w-4/5 pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
        :aria-label="entityProperty.name"
        v-bind:class="inputClasses"
        v-model="fieldValue"
        @focus="storeStartingValue"
        @blur="saveIfModified"
      >
        <option
          v-for="option in entityProperty.prop.variants"
          v-bind:key="option"
          >{{ option }}</option
        >
      </select>
      <ValidationWidget :value="fieldValue" :entityProperty="entityProperty" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState, mapGetters } from "vuex";
import _ from "lodash";

import { RootStore } from "@/store";
import { RegistryProperty } from "@/api/sdf/model/node";

import PropMixin from "./PropMixin";
import ValidationWidget from "@/components/ui/ValidationWidget.vue";

export default PropMixin.extend({
  name: "PropEnum",
  components: {
    ValidationWidget,
  },
  computed: {
    ...mapState({
      editorMode: (state: any): RootStore["editor"]["mode"] =>
        state.editor.mode,
    }),
  },
});
</script>
