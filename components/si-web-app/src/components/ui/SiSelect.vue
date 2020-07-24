<template>
  <div class="relative inline-block w-full" v-bind:class="textClasses">
    <select
      class="block w-full px-2 py-1 pr-8 leading-tight text-gray-400 border border-gray-800 border-solid shadow appearance-none focus:outline-none select"
      @change="selected"
    >
      <option
        v-for="(option, index) of options"
        v-bind:key="index"
        :value="option.value"
        :selected="isSelected(option.value, value)"
        class="mt-1"
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
import Vue from "vue";
import { ChevronDownIcon } from "vue-feather-icons";

export interface SelectPropsOption {
  value: string | null | Object;
  label: string;
}

export interface SelectProps {
  size: "xs" | "sm" | "base" | "lg";
  options: SelectPropsOption[];
  value: string | null | Object | number;
}

export default Vue.extend({
  name: "SiSelect",
  components: {
    ChevronDownIcon,
  },
  props: {
    name: String,
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

      if (newValue === null) {
        isSelected = optionValue === newValue;
      } else if (newValue.id) {
        isSelected = optionValue == newValue.id;
      } else {
        isSelected = optionValue == newValue;
      }
      return isSelected;
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
