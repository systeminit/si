<template>
  <div class="flex items-center mt-2" v-if="fieldValue || editorMode == 'edit'">
    <div class="w-40 px-2 text-sm leading-tight text-right text-white">
      {{ entityProperty.name }}
    </div>

    <div
      v-if="editorMode == 'view'"
      class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
      v-bind:class="textClasses"
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
    <div
      class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
      v-else-if="editorMode == 'edit'"
    >
      <template v-if="entityProperty.repeated">
        <div v-for="option in this.options" :key="option.key">
          <input
            type="checkbox"
            :id="option.key"
            :value="option.value"
            v-bind:class="inputClasses"
            v-model="fieldValue"
          />
          <label
            class="pl-2 text-sm leading-tight text-white input-label"
            :for="option.key"
            :aria-label="option"
          >
            {{ option.key }}
          </label>
        </div>
      </template>
      <template v-else>
        <select
          class="w-4/5 pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none"
          v-bind:class="inputClasses"
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
      <ValidationWidget :value="fieldValue" :entityProperty="entityProperty" />
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
import ValidationWidget from "@/components/ui/ValidationWidget.vue";

export default PropMixin.extend({
  name: "PropSelect",
  components: {
    ValidationWidget,
  },
  methods: {
    labelForValue(value: string): string {
      const option = _.find(this.options, ["value", value]);
      if (option) {
        return option.key;
      } else {
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
        // @ts-ignore
        return this.entityProperty.prop.options;
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
