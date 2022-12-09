<template>
  <div>
    <label
      :for="props.id"
      class="block text-sm font-medium text-neutral-800 dark:text-neutral-50"
    >
      {{ props.title }} <span v-if="required">(required)</span>
    </label>

    <div class="mt-1 w-full relative">
      <input
        :id="props.id"
        v-model="inputValue"
        :data-test="props.id"
        type="checkbox"
        :name="props.id"
        :autocomplete="props.id"
        :aria-invalid="inError"
        :disabled="props.disabled"
        :indeterminate.prop="isIndeterminate"
        required
        :class="
          clsx(
            'appearance-none block px-5 py-4 border border-neutral-300 dark:border-neutral-500 rounded-sm shadow-xs focus:outline-none sm:text-sm dark:bg-shade-100 bg-neutral-50 indeterminate:bg-neutral-500 indeterminate:dark:bg-neutral-500',
            inError
              ? 'border-destructive-400 focus:ring-destructive-400 focus:border-destructive-400 checked:border-destructive-400 indeterminate:border-destructive-400'
              : 'border-neutral-600 checked:dark:border-neutral-600 checked:border-neutral-200 indeterminate:dark:border-neutral-500 indeterminate:border-neutral-500',
          )
        "
        @blur="setDirty"
      />
      <div
        v-if="inError"
        class="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none text-destructive-400"
      >
        <Icon name="exclamation-circle" size="sm" />
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

<script setup lang="ts">
import { computed, toRef } from "vue";
import _ from "lodash";
import clsx from "clsx";
import SiValidation from "@/atoms/SiValidation.vue";
import { useValidations, ValidatorArray } from "@/utils/input_validations";
import Icon from "@/ui-lib/icons/Icon.vue";

const props = defineProps<{
  modelValue?: boolean;
  title: string;
  id: string;
  description?: string;

  validations?: ValidatorArray;
  required?: boolean;
  alwaysValidate?: boolean;

  docLink?: string;

  disabled?: boolean;
}>();

const emit = defineEmits(["update:modelValue", "error", "blur"]);

const alwaysValidate = toRef(props, "alwaysValidate", false);

const { inError, reallyDirty, setDirty, setInError } = useValidations(
  alwaysValidate,
  () => emit("blur", inputValue),
  (inError: boolean) => emit("error", inError),
);

const inputValue = computed<boolean | undefined>({
  get() {
    return props.modelValue;
  },
  set(value) {
    emit("update:modelValue", value);
  },
});

const isIndeterminate = computed(() => _.isUndefined(props.modelValue));
</script>
