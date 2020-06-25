<template>
  <div class="flex">
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
      <select
        class="bg-gray-800 border text-gray-400 text-sm px-4 leading-tight focus:outline-none"
        :aria-label="entityProperty.name"
        v-model="fieldValue"
      >
        <option
          v-for="option in entityProperty.prop.variants"
          v-bind:key="option"
          >{{ option }}</option
        >
      </select>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState, mapGetters } from "vuex";
import _ from "lodash";

import { RootStore } from "@/store";
import { EntityProperty } from "@/store/modules/entity";

export default Vue.extend({
  name: "PropEnum",
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
        await this.$store.dispatch("editor/setEditValue", {
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
