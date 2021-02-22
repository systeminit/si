<template>
  <div class="flex items-center mt-2" v-if="currentValue || editMode">
    <div class="w-40 px-2 text-sm leading-tight text-right text-white">
      {{ registryProperty.name }}
    </div>

    <div
      v-if="!editMode"
      class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400"
      v-bind:class="textClasses"
    >
      <template v-if="registryProperty.repeated">
        <ol>
          <li v-for="(field, index) of currentValue" :key="index">
            {{ labelForValue(field) }}
          </li>
        </ol>
      </template>
      <template v-else>
        {{ currentValue }}
      </template>
    </div>
    <div class="w-4/5 pl-2 mr-2 text-sm leading-tight text-gray-400" v-else>
      <template v-if="registryProperty.repeated">
        <div v-for="option in this.options" :key="option.key">
          <input
            type="checkbox"
            :id="option.key"
            :value="option.value"
            v-bind:class="inputClasses"
            v-model="currentValue"
            @focus="storeStartingValue"
            @blur="saveIfModified"
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
          :data-cy="
            `editor-property-viewer-prop-${registryProperty.path.join('-')}`
          "
          v-bind:class="inputClasses"
          :aria-label="registryProperty.name"
          v-model="currentValue"
          @focus="storeStartingValue"
          @blur="saveIfModified"
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
      <ValidationWidget
        :value="currentValue"
        :registryProperty="registryProperty"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState, mapGetters } from "vuex";
import _ from "lodash";

import PropMixin from "./PropMixin";
import ValidationWidget from "@/atoms/ValidationWidget.vue";

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
    options(): { key: string; value: string }[] {
      // @ts-ignore - we know its a PropSelect
      if (this.registryProperty.prop.optionsFromType) {
        return this.$store.getters["entity/optionsFromType"](
          // @ts-ignore - we still know!
          this.registryProperty.prop.optionsFromType,
        );
      } else {
        // @ts-ignore
        return this.registryProperty.prop.options;
      }
    },
  },
});
</script>
