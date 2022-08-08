<template>
  <div>
    <label
      :for="props.id"
      v-if="props.title"
      class="block text-sm font-medium"
      :class="titleClasses"
    >
      {{ props.title }} <span v-if="required">(required)</span>
    </label>

    <div class="mt-1 w-full relative">
      <textarea
        v-if="props.textArea"
        :id="id"
        :type="type"
        :aria-required="required"
        :placeholder="placeholder"
        :value="modelValue"
        :data-test="id"
        class="appearance-none block h-24 bg-gray-900 text-gray-100 w-full px-3 py-2 border border-gray-600 rounded-sm shadow-sm placeholder-gray-900 focus:outline-none focus:ring-indigo-200 focus:border-indigo-200 sm:text-sm"
        @input="valueChanged"
      />
      <input
        v-else
        :id="props.id"
        v-model="inputValue"
        :data-test="props.id"
        :placeholder="props.placeholder"
        :type="type"
        :name="props.id"
        :autocomplete="props.id"
        :aria-invalid="inError"
        :disabled="props.disabled"
        required
        class="appearance-none block w-full px-3 py-2 border rounded-sm shadow-sm focus:outline-none sm:text-sm"
        :class="textBoxClasses"
        v-bind="$attrs"
        @blur="setDirty"
      />

      <div
        v-if="inError"
        class="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none"
      >
        <ExclamationCircleIcon
          class="h-5 w-5 text-destructive-400"
          aria-hidden="true"
        />
      </div>
    </div>

    <p v-if="props.docLink" class="mt-2 text-xs text-action-500">
      <a :href="props.docLink" target="_blank" class="hover:underline">
        Documentation
      </a>
    </p>

    <p v-if="props.description" class="mt-2 text-xs text-neutral-300">
      {{ props.description }}
    </p>

    <SiValidation
      :value="String(inputValue)"
      :validations="validations"
      :required="required"
      :dirty="reallyDirty"
      class="mt-2"
      @errors="setInError($event)"
    />
  </div>
</template>

<script lang="ts">
export default {
  inheritAttrs: false,
};
</script>

<script setup lang="ts">
import { ExclamationCircleIcon } from "@heroicons/vue/solid";
import SiValidation, {
  ValidatorArray,
  ErrorsArray,
} from "@/atoms/SiValidation.vue";
import { computed, ref } from "vue";
import _ from "lodash";

const props = defineProps<{
  modelValue: string;
  title?: string;
  id: string;
  description?: string;

  placeholder?: string;
  password?: boolean;

  validations?: ValidatorArray;
  required?: boolean;
  alwaysValidate?: boolean;

  docLink?: string;

  textArea?: boolean;
  disabled?: boolean;
  loginMode?: boolean;
}>();

const emit = defineEmits(["update:modelValue", "error", "blur"]);

const dirty = ref<boolean>(false);
const setDirty = () => {
  dirty.value = true;
  emit("blur", inputValue);
};

const reallyDirty = computed(() => {
  if (props.alwaysValidate) {
    return true;
  }
  return dirty.value;
});

const inError = ref<boolean>(false);
const setInError = (errors: ErrorsArray) => {
  let nextInError = false;
  if (errors.length == 1) {
    if (_.find(errors, (e) => e.id == "required")) {
      if (dirty.value) {
        nextInError = true;
      }
    } else {
      nextInError = true;
    }
  } else if (errors.length > 1) {
    nextInError = true;
  }
  inError.value = nextInError;
  emit("error", inError);
};

const inputValue = computed<string>({
  get() {
    return props.modelValue;
  },
  set(value) {
    emit("update:modelValue", value);
  },
});

const textBoxClasses = computed((): Record<string, boolean> => {
  const results: Record<string, boolean> = {
    "placeholder-neutral-400": true,
  };

  if (props.loginMode) {
    results["bg-shade-100"] = true;
    results["text-neutral-100"] = true;
    results["disabled:border-neutral-100"] = true;
  } else {
    results["bg-neutral-50"] = true;
    results["border-neutral-300"] = true;
    results["dark:border-neutral-600"] = true;
    results["dark:bg-neutral-700"] = true;
  }

  if (inError.value) {
    results["border-destructive-400"] = true;
    results["focus:ring-destructive-400"] = true;
    results["focus:border-destructive-400"] = true;
  } else {
    results["border-neutral-600"] = true;
    results["focus:ring-active-200"] = true;
    results["focus:border-active-200"] = true;
  }
  return results;
});

const titleClasses = computed((): Record<string, boolean> => {
  if (props.loginMode) {
    return {
      "text-neutral-50": true,
    };
  }
  return {};
});

const type = computed((): string => {
  if (props.password) {
    return "password";
  }
  return "text";
});

const valueChanged = (event: Event) => {
  const element = event.currentTarget as HTMLInputElement;
  emit("update:modelValue", element.value);
};
</script>
