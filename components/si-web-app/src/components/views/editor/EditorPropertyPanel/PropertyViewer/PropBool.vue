<template>
  <div class="flex items-center mt-2" v-if="fieldValue || editorMode == 'edit'">
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
      class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400 justiy-start"
      v-else-if="editorMode == 'edit'"
    >
      <input
        class="pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
        :data-cy="
          `editor-property-viewer-prop-${entityProperty.path.join('-')}`
        "
        v-bind:class="inputClasses"
        type="checkbox"
        :aria-label="entityProperty.name"
        v-model="fieldValue"
        @focus="storeStartingValue"
        @blur="saveIfModified"
      />
      <ValidationWidget :value="fieldValue" :entityProperty="entityProperty" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { Store, mapState, mapGetters } from "vuex";
import _ from "lodash";

import { RootStore } from "@/store";
import { RegistryProperty } from "@/api/sdf/model/node";

import PropMixin from "./PropMixin";
import ValidationWidget from "@/components/ui/ValidationWidget.vue";

export default PropMixin.extend({
  name: "PropBool",
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
