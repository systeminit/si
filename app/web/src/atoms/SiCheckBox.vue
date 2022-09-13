<template>
  <label :for="props.id" class="block text-sm font-medium text-neutral-200">
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
          'appearance-none block px-3 py-2 border rounded-sm shadow-sm focus:outline-none sm:text-sm checked:bg-neutral-900 bg-neutral-600 indeterminate:bg-neutral-900',
          inError
            ? 'border-destructive-400 focus:ring-destructive-400 focus:border-destructive-400 checked:border-destructive-400 indeterminate:border-destructive-400'
            : 'border-neutral-600 focus:ring-action-200 focus:border-action-200 checked:border-neutral-600 indeterminate:border-neutral-600',
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
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import _ from "lodash";
import clsx from "clsx";
import SiValidation, {
  ValidatorArray,
  ErrorsArray,
} from "@/atoms/SiValidation.vue";
import Icon from "@/ui-lib/Icon.vue";

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
  if (errors.length === 1) {
    if (_.find(errors, (e) => e.id === "required")) {
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
