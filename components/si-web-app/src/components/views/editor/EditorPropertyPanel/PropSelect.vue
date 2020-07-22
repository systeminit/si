<template>
  <div class="flex">
    <div class="input-label text-sm leading-tight pl-2 text-white">
      {{ entityProperty.name }}:
    </div>

    <div
      v-if="editorMode == 'view'"
      class="text-sm leading-tight text-gray-400 pl-2 h-5"
    >
      <template v-if="entityProperty.repeated">
        <ol>
          <li v-for="(field, index) of fieldValue" :key="index">
            {{ labelForValue(field) }}
          </li>
        </ol>
      </template>
      <template v-else>
        {{ fieldValue }}
      </template>
    </div>
    <div v-else-if="editorMode == 'edit'">
      <template v-if="entityProperty.repeated">
        <div v-for="option in this.options" :key="option.key">
          <input
            type="checkbox"
            :id="option.key"
            :value="option.value"
            v-model="fieldValue"
          />
          <label
            class="pl-2 text-sm text-white leading-tight input-label"
            :for="option.key"
            :aria-label="option"
          >
            {{ option.key }}
          </label>
        </div>
      </template>
      <template v-else>
        <select
          class="bg-gray-800 border text-gray-400 text-sm px-4 leading-tight focus:outline-none"
          :aria-label="entityProperty.name"
          v-model="fieldValue"
        >
          <option
            v-for="option in this.options"
            :key="option.key"
            :value="option.value"
          >
            {{ option.key }}
          </option>
        </select>
      </template>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState, mapGetters } from "vuex";
import _ from "lodash";

import { RootStore } from "@/store";
import { RegistryProperty, debouncedSetFieldValue } from "@/store/modules/node";

export default Vue.extend({
  name: "PropSelect",
  props: {
    entityProperty: Object as () => RegistryProperty,
  },
  methods: {
    labelForValue(value: string): string {
      const option = _.find(this.options, ["value", value]);
      if (option) {
        return option.key;
      } else {
        console.log("Cannot find the option for this value!", {
          value,
          options: this.options,
        });
        return value;
      }
    },
  },
  computed: {
    ...mapState({
      editorMode: (state: any): RootStore["editor"]["mode"] =>
        state.editor.mode,
    }),
    options(): { key: string; value: string }[] {
      // @ts-ignore - we know its a PropSelect
      if (this.entityProperty.prop.optionsFromType) {
        return this.$store.getters["entity/optionsFromType"](
          // @ts-ignore - we still know!
          this.entityProperty.prop.optionsFromType,
        );
      } else {
        return this.options;
      }
    },
    fieldValue: {
      get(): string | string[] {
        let value = this.$store.getters["node/getFieldValue"](
          this.entityProperty.path,
        );
        if (this.entityProperty.repeated) {
          if (value) {
            return value;
          } else {
            return [];
          }
        } else {
          return value;
        }
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
