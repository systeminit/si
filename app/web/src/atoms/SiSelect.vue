<template>
  <div class="relative inline-block w-full" :class="textClasses">
    <select
      :id="id"
      class="block w-full px-2 py-1 pr-8 leading-tight border border-solid shadow appearance-none focus:outline-none"
      :class="selectorStyling()"
      :disabled="disabled"
      :aria-name="id"
      :data-test="dataTest"
      @change="selected"
      @keypress.space.prevent
    >
      <option
        v-for="(option, index) of options"
        :key="index"
        :value="option.value"
        :selected="isSelected(option.value, modelValue)"
      >
        {{ option.label }}
      </option>
    </select>
    <div
      class="absolute inset-y-0 right-0 flex items-center px-2 text-gray-400 pointer-events-none"
    >
      <VueFeather type="chevron-down" :size="iconSize" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { PropType, computed } from "vue";
import VueFeather from "vue-feather";

export interface SelectPropsOption {
  value: unknown;
  label: unknown;
}

export interface SelectProps {
  size: "xs" | "sm" | "base" | "lg";
  options: SelectPropsOption[];

  // TODO: fix below
  // eslint-disable-next-line @typescript-eslint/ban-types
  value: string | null | Object | number;
}

const props = defineProps({
  styling: {
    type: Object as PropType<Record<string, unknown>>,
    default: null,
  },
  id: {
    type: String,
    required: true,
  },
  size: {
    type: String as () => SelectProps["size"],
    default: "base",
  },
  options: {
    type: Array as () => SelectProps["options"],
    required: true,
  },
  modelValue: {
    type: [String, Object, Number],
    default: "",
    required: false,
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  valueAsNumber: {
    type: Boolean,
    default: false,
  },
  dataTest: {
    type: String,
    default: "",
    required: false,
  },
});
const emits = defineEmits(["update:modelValue"]);

const selected = (event: Event) => {
  // https://v3.vuejs.org/guide/typescript-support.html#typing-event-handlers
  const value = (event.target as HTMLInputElement).value;
  if (value === "") {
    emits("update:modelValue", null);
  } else {
    if (props.valueAsNumber) {
      emits("update:modelValue", Number(value));
    } else {
      emits("update:modelValue", value);
    }
  }
};

const isSelected = (
  optionValue: unknown,
  newValue: string | number | Record<string, unknown>,
) => {
  let isSelected = false;

  if (newValue === null || newValue === undefined) {
    isSelected = optionValue === null;
  } else if (typeof newValue === "string" || typeof newValue === "number") {
    isSelected = optionValue == newValue;
  } else if (newValue.id) {
    isSelected = optionValue == newValue.id;
  }

  return isSelected;
};

const textClasses = computed(() => {
  const result: Record<string, boolean> = {};
  const textSize = `text-${props.size}`;
  result[textSize] = true;
  return result;
});

const selectorStyling = () => {
  const classes: Record<string, boolean> = {};
  const textSize = `text-${props.size}`;
  classes[textSize] = true;
  if (!props.styling) {
    classes["text-gray-400"] = true;
    classes["select"] = true;
  } else {
    return props.styling;
  }
  return classes;
};

const iconSize = computed(() => {
  switch (props.size) {
    case "xs":
      return "0.8rem";
    case "sm":
      return "1.0rem";
    case "base":
      return "1.5rem";
    case "lg":
      return "1.8rem";
    default:
      return "1.0rem";
  }
});
</script>

<style scoped>
.select {
  background-color: #2d3748;
  border-color: #485359;
}
</style>
