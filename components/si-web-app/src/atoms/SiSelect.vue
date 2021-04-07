<template>
  <div class="relative inline-block w-full" v-bind:class="textClasses">
    <select
      class="block w-full px-2 py-1 pr-8 leading-tight border border-solid shadow appearance-none focus:outline-none"
      :class="selectorStyling()"
      :disabled="disabled"
      :data-testid="id"
      :aria-name="id"
      :id="id"
      @change="selected"
      @keypress.space.prevent
    >
      <option
        v-for="(option, index) of options"
        v-bind:key="index"
        :value="option.value"
        :selected="isSelected(option.value, value)"
      >
        {{ option.label }}
      </option>
    </select>
    <div
      class="absolute inset-y-0 right-0 flex items-center px-2 text-gray-400 pointer-events-none"
    >
      <ChevronDownIcon v-bind:size="iconSize" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { ChevronDownIcon } from "vue-feather-icons";

export interface SelectPropsOption {
  value: string | null | Object;
  label: string;
}

export interface SelectProps {
  size: "xs" | "sm" | "base" | "lg";
  options: SelectPropsOption[];
  value: string | null | Object | number;
  dataCy: string;
}

export default Vue.extend({
  name: "SiSelect",
  components: {
    ChevronDownIcon,
  },
  props: {
    name: String,
    styling: {
      type: Object as PropType<Record<string, any>>,
      default: null,
    },
    id: {
      type: String,
    },
    size: {
      type: String as () => SelectProps["size"],
      default: "base",
    },
    options: {
      type: Array as () => SelectProps["options"],
    },
    value: {
      type: [String, Object, Number],
    },
    disabled: {
      type: Boolean,
      default: false,
    },
    dataCy: {
      type: String,
    },
  },
  methods: {
    selected(event: any): void {
      if (event.target.value === "") {
        this.$emit("input", null);
      } else {
        this.$emit("input", event.target.value);
      }
    },
    isSelected(optionValue: any, newValue: any): boolean {
      let isSelected = false;

      if (newValue === null || newValue === undefined) {
        isSelected = optionValue === null;
      } else if (newValue.id) {
        isSelected = optionValue == newValue.id;
      } else {
        isSelected = optionValue == newValue;
      }
      return isSelected;
    },
    selectorStyling(): Record<string, any> {
      let classes: Record<string, any> = {};
      if (!this.styling) {
        classes["text-gray-400"] = true;
        classes["select"] = true;
      } else {
        return this.styling;
      }
      return classes;
    },
  },
  computed: {
    textClasses(): Record<string, boolean> {
      const result: Record<string, boolean> = {};
      const textSize = `text-${this.size}`;
      result[textSize] = true;
      return result;
    },
    iconSize(): string {
      switch (this.size) {
        case "xs":
          return "0.8x";
        case "sm":
          return "1.0x";
        case "base":
          return "1.5x";
        case "lg":
          return "1.8x";
        default:
          return "1.0x";
      }
    },
  },
});
</script>

<style scoped>
.select {
  background-color: #2d3748;
  border-color: #485359;
}
</style>
