<template>
  <div>
    <div class="flex flex-row">
      <label
        :for="props.id"
        class="block text-sm font-medium text-neutral-800 dark:text-neutral-50"
      >
        {{ props.title }} <span v-if="required">(required)</span>
      </label>

      <Switch
        :id="props.id"
        v-model="inputValue"
        :disabled="props.disabled"
        :class="
          inputValue ? 'bg-success-600' : 'bg-neutral-400 dark:bg-neutral-500'
        "
        class="relative inline-flex h-5 w-8 items-center rounded-full ml-2"
        :aria-invalid="inError"
        @blur="setDirty"
      >
        <span
          :class="inputValue ? 'translate-x-4' : 'translate-x-1'"
          class="inline-block h-3 w-3 transform rounded-full bg-white transition"
        />
      </Switch>

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
      class="mt-2"
      @errors="setInError($event)"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, toRef } from "vue";
import * as _ from "lodash-es";
import { Switch } from "@headlessui/vue";
import { Icon } from "@si/vue-lib/design-system";
import { useValidations, ValidatorArray } from "@/utils/input_validations";
import SiValidation from "./SiValidation.vue";

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

const { inError, setDirty, setInError } = useValidations(
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

// TODO: maybe want this to be a thing again?
// const isIndeterminate = computed(() => _.isUndefined(props.modelValue));
</script>
