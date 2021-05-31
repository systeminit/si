<template>
  <div class="relative inline-block w-full" v-bind:class="textClasses">
    <input
      :type="type"
      :aria-required="required"
      :id="id"
      :placeholder="placeholder"
      :value="value"
      :data-testid="id"
      class="block w-full px-2 py-1 pr-8 leading-tight text-gray-400 border border-gray-800 border-solid shadow shadow-inner focus:outline-none bg-blueGray-700"
      @input="valueChanged"
      v-if="!isTextArea"
      @keyup.stop
      @keydown.stop
    />

    <textarea
      :type="type"
      :aria-required="required"
      :id="id"
      :placeholder="placeholder"
      :value="value"
      :data-testid="id"
      class="block w-full h-24 px-2 py-1 pr-8 leading-tight text-gray-400 border border-gray-800 border-solid shadow shadow-inner resize-y focus:outline-none bg-blueGray-700"
      @input="valueChanged"
      v-if="isTextArea"
    />
    <div
      class="absolute inset-y-0 right-0 flex items-center px-2 text-gray-400 pointer-events-none"
    >
      <TypeIcon v-bind:size="iconSize" v-if="type == 'text' && isShowType" />
      <KeyIcon
        v-bind:size="iconSize"
        v-else-if="type == 'password' && isShowType"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { TypeIcon, KeyIcon } from "vue-feather-icons";

export interface TextBoxProps {
  size: "xs" | "sm" | "base" | "lg";
  value: string;
  placholder: string;
  name: string;
  dataTestId: string;
}

export default Vue.extend({
  name: "SiTextBox",
  components: {
    TypeIcon,
    KeyIcon,
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
    id: {
      type: String,
    },
    required: {
      type: Boolean,
      default: false,
    },
    type: {
      type: String,
      default: "text",
    },
    isTextArea: {
      type: Boolean,
      default: false,
      required: false,
    },
    isShowType: {
      type: Boolean,
      default: false,
      required: false,
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
          return "0.5x";
        case "sm":
          return "0.8x";
        case "base":
          return "1.0x";
        case "lg":
          return "1.2x";
        default:
          return "1.0x";
      }
    },
  },
});
</script>
