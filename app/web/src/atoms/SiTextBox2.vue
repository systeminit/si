<template>
  <label :for="props.id" class="block text-sm font-medium text-gray-200">
    {{ props.title }}
  </label>

  <div class="mt-1 w-full relative">
    <input
      :id="props.id"
      :value="modelValue"
      :data-test="props.id"
      :placeholder="props.placeholder"
      :type="type"
      :name="props.id"
      :autocomplete="props.id"
      :aria-invalid="props.error !== undefined"
      :aria-describedby="ariaDescribedBy"
      required
      class="appearance-none block bg-gray-900 text-gray-100 w-full px-3 py-2 border rounded-sm shadow-sm placeholder-gray-400 focus:outline-none sm:text-sm"
      :class="textBoxClasses"
      @input="valueChanged"
    />
    <div
      v-if="props.error"
      class="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none"
    >
      <ExclamationCircleIcon class="h-5 w-5 text-red-400" aria-hidden="true" />
    </div>
  </div>

  <p :class="descriptionClasses">
    {{ props.description }}
  </p>
  <p v-if="props.error" :id="props.error.id" class="mt-2 text-xs text-red-400">
    {{ props.error.message }}
  </p>
</template>

<script setup lang="ts">
import { ExclamationCircleIcon } from "@heroicons/vue/solid";
import { computed } from "vue";

const props = defineProps<{
  modelValue: string;
  title: string;
  id: string;
  description: string;

  error?: {
    id: string;
    message: string;
  };

  placeholder?: string;
  password?: boolean;
}>();

const textBoxClasses = computed((): Record<string, boolean> => {
  if (props.error) {
    return {
      "border-red-400": true,
      "focus:ring-red-400": true,
      "focus:border-red-400": true,
    };
  }
  return {
    "border-gray-600": true,
    "focus:ring-indigo-200": true,
    "focus:border-indigo-200": true,
  };
});

const descriptionClasses = computed((): Record<string, boolean> => {
  if (props.error) {
    return {
      "mt-2": true,
      "text-xs": true,
      "text-gray-300": true,
    };
  }
  // Add extra padding to the bottom where the error message would have been.
  return {
    "mt-2": true,
    "text-xs": true,
    "text-gray-300": true,
    "mb-6": true,
  };
});

const type = computed((): string => {
  if (props.password) {
    return "password";
  }
  return "text";
});

const ariaDescribedBy = computed((): string | undefined => {
  if (props.error) {
    return props.error.id;
  }
  return undefined;
});

const emit = defineEmits(["update:modelValue"]);
const valueChanged = (event: Event) => {
  const element = event.currentTarget as HTMLInputElement;
  emit("update:modelValue", element.value);
};
</script>
