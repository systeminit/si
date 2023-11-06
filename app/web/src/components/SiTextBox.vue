<template>
  <div>
    <label
      v-if="title"
      :for="id"
      class="block text-sm font-medium"
      :class="titleClasses"
    >
      {{ title }}
      <span
        v-if="required && !formSettings.hideRequiredLabel"
        :class="formSettings.requiredLabelClasses"
        >{{ formSettings.requiredLabel }}</span
      >
    </label>

    <div class="mt-1 w-full relative">
      <textarea
        v-if="textArea"
        :id="id"
        :type="type"
        :aria-required="required"
        :placeholder="placeholder"
        :value="modelValue"
        :data-test="id"
        class="appearance-none block w-full py-2 border rounded-sm shadow-sm focus:outline-none sm:text-sm"
        :class="textBoxClasses"
        required
        :disabled="disabled"
        @input="valueChanged"
        @blur="setDirty"
      />
      <input
        v-else
        :id="id"
        v-model="inputValue"
        :data-test="id"
        :placeholder="placeholder"
        :type="type"
        :name="id"
        :autocomplete="id"
        :aria-invalid="inError"
        :disabled="disabled"
        required
        class="appearance-none block w-full py-2 border rounded-sm shadow-sm focus:outline-none sm:text-sm"
        :class="textBoxClasses"
        v-bind="$attrs"
        :passwordrules="
          password
            ? `minlength: ${minPasswordLength}; maxlength: ${maxPasswordLength}; required: lower; required: upper; required: digit; required: special;`
            : undefined
        "
        :minlength="password ? minPasswordLength : undefined"
        :maxlength="password ? maxPasswordLength : undefined"
        @blur="setDirty"
      />

      <div
        v-if="inError"
        class="absolute inset-y-0 right-0 pr-2 flex items-center text-destructive-400"
      >
        <Icon name="exclamation-circle" />
      </div>
    </div>

    <p v-if="docLink" class="mt-2 text-xs text-action-500">
      <a :href="docLink" target="_blank" class="hover:underline">
        Documentation
      </a>
    </p>

    <p v-if="description" class="mt-2 text-xs text-neutral-300">
      {{ description }}
    </p>

    <SiValidation
      :value="String(inputValue)"
      :validations="validations"
      class="mt-2"
      @errors="setInError($event)"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, PropType, toRefs } from "vue";
import * as _ from "lodash-es";
import { Icon } from "@si/vue-lib/design-system";
import { useFormSettings } from "@/utils/formSettings";
import { ValidatorArray, useValidations } from "@/utils/input_validations";
import SiValidation from "./SiValidation.vue";

const props = defineProps({
  modelValue: { type: String },
  title: String,
  id: { type: String, required: true },
  description: String,

  placeholder: String,
  password: Boolean,
  minPasswordLength: { type: Number, default: 8 },
  maxPasswordLength: { type: Number, default: 64 },

  validations: { type: Array as PropType<ValidatorArray> },
  required: Boolean,
  alwaysValidate: Boolean,

  docLink: String,

  textArea: Boolean,
  disabled: Boolean,
  loginMode: Boolean,
  renderAsPassword: { type: Boolean, default: false },
});

const { alwaysValidate } = toRefs(props);

const formSettings = useFormSettings();

const emit = defineEmits(["update:modelValue", "error", "blur"]);

const inputValue = computed<string>({
  get() {
    return props.modelValue ?? "";
  },
  set(value) {
    emit("update:modelValue", value);
  },
});

const { inError, setInError, setDirty } = useValidations(
  alwaysValidate,
  () => emit("blur", inputValue),
  (inError: boolean) => emit("error", inError),
);

const textBoxClasses = computed((): Record<string, boolean> => {
  const results: Record<string, boolean> = {
    "placeholder-neutral-400": true,
  };

  if (props.loginMode) {
    results["bg-shade-100"] = true;
    results["text-neutral-100"] = true;
    results["disabled:border-neutral-100"] = true;
    results["placeholder:italic"] = true;
    results["placeholder:text-xs"] = true;
  } else {
    results["bg-neutral-50"] = true;
    results["border-neutral-200"] = true;
    results["dark:border-neutral-600"] = true;
    results["dark:bg-neutral-900"] = true;
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
  if (props.password || props.renderAsPassword) {
    return "password";
  } else {
    return "text";
  }
});

const valueChanged = (event: Event) => {
  const element = event.currentTarget as HTMLInputElement;
  emit("update:modelValue", element.value);
};
</script>

<script lang="ts">
export default {
  inheritAttrs: false,
};
</script>
