<template>
  <div class="relative inline-block w-full" v-bind:class="textClasses">
    <input
      type="text"
      :aria-label="name"
      :placeholder="placeholder"
      :value="value"
      :data-cy="dataCy"
      class="block w-full px-2 py-1 pr-8 leading-tight text-gray-400 border border-gray-800 border-solid shadow focus:outline-none textbox"
      @input="valueChanged"
    />
    <div
      class="absolute inset-y-0 right-0 flex items-center px-2 text-gray-400 pointer-events-none"
    >
      <TypeIcon v-bind:size="iconSize" />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { TypeIcon } from "vue-feather-icons";

export interface TextBoxProps {
  size: "xs" | "sm" | "base" | "lg";
  value: string;
  placholder: string;
  name: string;
  dataCy: string;
}

export default Vue.extend({
  name: "SiTextBox",
  components: {
    TypeIcon,
  },
  props: {
    name: {
      type: String,
      required: true,
    },
    size: {
      type: String as () => TextBoxProps["size"],
      default: "base",
    },
    value: {
      type: String,
    },
    placeholder: {
      type: String,
    },
    dataCy: {
      type: String,
    },
  },
  methods: {
    valueChanged(event: any): void {
      this.$emit("input", event.target.value);
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
.textbox {
  background-color: #2d3748;
  border-color: #485359;
}
</style>
