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
        class="appearance-none text-sm leading-tight focus focus:outline-none input-bg-color border-none font-bold text-gray-400 pl-2 h-5"
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
import { EntityProperty } from "@/store/modules/entity";
import { debouncedFieldValueSet } from "@/store/modules/editor";

export default Vue.extend({
  name: "PropNumber",
  props: {
    entityProperty: Object as () => EntityProperty,
    editEntry: Object as () => Record<string, any>,
  },
  computed: {
    ...mapState({
      editorMode: (state: any): RootStore["editor"]["mode"] =>
        state.editor.mode,
    }),
    fieldValue: {
      get(): string {
        return this.$store.getters["editor/getEditValue"](
          this.entityProperty.path,
        );
      },
      async set(value: any) {
        // @ts-ignore - we know you're a number
        if (this.entityProperty.prop.numberKind == "int32") {
          value = parseInt(value);
        }
        debouncedFieldValueSet({
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
