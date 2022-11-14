<template>
  <label
    :for="props.id"
    class="block text-sm font-medium text-neutral-900 dark:text-neutral-50"
  >
    {{ props.title }} <span v-if="required">(required)</span>
  </label>

  <div class="mt-1 w-full relative">
    <SiSelect
      :id="props.id"
      v-model="inputValue"
      :options="props.options"
      :data-test="props.id"
      :disabled="props.disabled"
      required
      class="appearance-none block px-3 py-2 border rounded-sm shadow-sm focus:outline-none sm:text-sm bg-neutral-50 dark:bg-neutral-700 border-neutral-600"
      :class="boxClasses"
      @change="setDirty"
    />
    <div
      v-if="inError"
      class="absolute inset-y-0 right-0 pr-3 flex items-center text-destructive-400"
    >
      <Icon name="exclamation-circle" />
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
</template>

<script setup lang="ts">
import { computed, toRef } from "vue";
import _ from "lodash";
import SiValidation from "@/atoms/SiValidation.vue";
import { ValidatorArray, useValidations } from "@/utils/input_validations";
import SiSelect from "@/atoms/SiSelect.vue";
import { LabelList } from "@/api/sdf/dal/label_list";
import Icon from "@/ui-lib/icons/Icon.vue";

const props = defineProps<{
  modelValue: string | number | null;
  options: LabelList<string | number>;
  title: string;
  id: string;
  description?: string;

  validations?: ValidatorArray;
  required?: boolean;
  alwaysValidate?: boolean;

  docLink?: string;

  disabled?: boolean;
}>();

const alwaysValidate = toRef(props, "alwaysValidate", false);

const emit = defineEmits(["update:modelValue", "error", "change"]);

const inputValue = computed<string | number | null>({
  get() {
    return props.modelValue;
  },
  set(value) {
    emit("update:modelValue", value);
  },
});

const { inError, reallyDirty, setDirty, setInError } = useValidations(
  alwaysValidate,
  () => emit("change", inputValue),
  (inError: boolean) => emit("error", inError),
);

const boxClasses = computed((): Record<string, boolean> => {
  if (inError.value) {
    return {
      "border-destructive-400": true,
      "focus:ring-destructive-400": true,
      "focus:border-destructive-400": true,
    };
  }
  return {
    "border-neutral-600": true,
    "focus:ring-action-200": true,
    "focus:border-action-200": true,
  };
});
</script>
