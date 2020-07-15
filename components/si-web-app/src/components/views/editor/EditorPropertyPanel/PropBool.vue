<template>
  <div class="flex" v-if="fieldValue || editorMode == 'edit'">
    <div class="input-label text-sm leading-tight pl-2 text-white">
      {{ entityProperty.name }}:
    </div>

    <div
      v-if="editorMode == 'view'"
      class="text-sm leading-tight text-gray-400 pl-2 h-5"
    >
      {{ fieldValue }}
    </div>

    <div v-else-if="editorMode == 'edit'">
      <input
        class="text-sm leading-tight focus:outline-none input-bg-color border-none text-gray-400 ml-4 pl-2 h-5"
        type="checkbox"
        :aria-label="entityProperty.name"
        v-model="fieldValue"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { Store, mapState, mapGetters } from "vuex";
import _ from "lodash";

import { RootStore } from "@/store";
import { RegistryProperty, debouncedSetFieldValue } from "@/store/modules/node";

export default Vue.extend({
  name: "PropBool",
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
.property-editor-bg-color {
  background-color: #212324;
}

.property-title-bg-color {
  background-color: #292c2d;
}

.input-bg-color {
  background-color: #25788a;
}

.input-label {
  @apply pr-2 text-sm text-gray-400 text-right w-40;
}

input[type="number"]::-webkit-inner-spin-button,
input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
</style>
