<template>
  <div class="relative inline-block w-full" :class="textClasses">
    <input
      v-if="!isTextArea"
      :id="id"
      :type="type"
      :aria-required="required"
      :value="modelValue"
      :data-test="id"
      class="appearance-none block bg-gray-900 text-gray-100 w-full px-3 py-2 border border-gray-600 rounded-sm shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-200 focus:border-indigo-200 sm:text-sm"
      @input="valueChanged"
    />

    <textarea
      v-if="isTextArea"
      :id="id"
      :type="type"
      :aria-required="required"
      :placeholder="placeholder"
      :value="modelValue"
      :data-test="id"
      class="appearance-none block h-24 bg-gray-900 text-gray-100 w-full px-3 py-2 border border-gray-600 rounded-sm shadow-sm placeholder-gray-900 focus:outline-none focus:ring-indigo-200 focus:border-indigo-200 sm:text-sm"
      @input="valueChanged"
    />
    <div
      class="absolute inset-y-0 right-0 flex items-center px-2 text-gray-400 pointer-events-none"
    >
      <VueFeather
        v-if="type === 'text' && isShowType"
        type="type"
        :size="iconSize"
      />
      <VueFeather
        v-else-if="type === 'password' && isShowType"
        type="key"
        :size="iconSize"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import VueFeather from "vue-feather";

const props = defineProps({
  size: {
    type: String as () => "xs" | "sm" | "base" | "lg",
    default: "base",
  },
  modelValue: {
    type: String,
    default: "",
  },
  placeholder: {
    type: String,
    required: true,
  },
  id: {
    type: String,
    required: true,
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
});
const emit = defineEmits(["update:modelValue"]);

const valueChanged = (event: Event) => {
  const element = event.currentTarget as HTMLInputElement;
  emit("update:modelValue", element.value);
};

const textClasses = computed(() => {
  const result: Record<string, boolean> = {};
  const textSize = `text-${props.size}`;
  result[textSize] = true;
  return result;
});

const iconSize = computed(() => {
  switch (props.size) {
    case "xs":
      return "0.5rem";
    case "sm":
      return "0.8rem";
    case "base":
      return "1.0rem";
    case "lg":
      return "1.2rem";
    default:
      return "1.0rem";
  }
});
</script>
