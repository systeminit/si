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
      class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
      v-else-if="editorMode == 'edit'"
    >
      <input
        class="w-4/5 pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
        v-bind:class="inputClasses"
        type="number"
        :aria-label="entityProperty.name"
        v-model="fieldValue"
        placeholder="number"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState, mapGetters } from "vuex";
import _ from "lodash";

import { RootStore } from "@/store";
import { RegistryProperty, debouncedSetFieldValue } from "@/store/modules/node";

import PropMixin from "./PropMixin";

export default PropMixin.extend({
  name: "PropNumber",
  props: {
    entityProperty: Object as () => RegistryProperty,
  },
  computed: {
    ...mapState({
      editorMode: (state: any): RootStore["editor"]["mode"] =>
        state.editor.mode,
    }),
    fieldValue: {
      get(): string {
        return this.$store.getters["node/getFieldValue"](
          this.entityProperty.path,
        );
      },
      async set(value: any) {
        // @ts-ignore - we know you're a number
        if (this.entityProperty.prop.numberKind == "int32") {
          value = parseInt(value);
        }
        debouncedSetFieldValue({
          store: this.$store,
          path: this.entityProperty.path,
          value,
        });
      },
    },
  },
});
</script>

<style scoped>
input[type="number"]::-webkit-inner-spin-button,
input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
</style>
