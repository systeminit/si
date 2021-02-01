<template>
  <div class="flex items-center mt-2" v-if="fieldValue || editorMode == 'edit'">
    <div class="w-40 px-2 text-sm leading-tight text-right text-white">
      {{ entityProperty.name }}
    </div>

    <pre
      v-if="editorMode == 'view'"
      class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
      v-bind:class="textClasses"
    >
      {{ fieldValue }}
    </pre>
    <div
      class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
      v-else-if="editorMode == 'edit'"
    >
      <textarea
        class="w-4/5 pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
        v-bind:class="inputClasses"
        :aria-label="entityProperty.name"
        v-model="fieldValue"
        placeholder="text area"
        rows="16"
        @focus="storeStartingValue"
        @blur="saveIfModified"
      >
      </textarea>
      <ValidationWidget :value="fieldValue" :entityProperty="entityProperty" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { Store, mapState, mapGetters } from "vuex";
import _ from "lodash";

import { RootStore } from "@/store";

import PropMixin from "./PropMixin";
import ValidationWidget from "@/components/ui/ValidationWidget.vue";

interface Data {
  originalValue: any;
}

export default PropMixin.extend({
  name: "PropTextArea",
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

<style scoped>
.property-editor-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292c2d;
}
</style>
