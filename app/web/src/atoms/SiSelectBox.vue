<template>
  <label :for="props.id" class="block text-sm font-medium text-gray-200">
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
      class="appearance-none block px-3 py-2 border rounded-sm shadow-sm focus:outline-none sm:text-sm bg-gray-600"
      :class="boxClasses"
      @change="setDirty"
    />
    <div
      v-if="inError"
      class="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none"
    >
      <ExclamationCircleIcon class="h-5 w-5 text-red-400" aria-hidden="true" />
    </div>
  </div>

  <p v-if="props.docLink" class="mt-2 text-xs text-blue-300">
    <a :href="props.docLink" target="_blank" class="hover:underline">
      Documentation
    </a>
  </p>

  <p v-if="props.description" class="mt-2 text-xs text-gray-300">
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
import { ExclamationCircleIcon } from "@heroicons/vue/solid";
import SiValidation, {
  ValidatorArray,
  ErrorsArray,
} from "@/atoms/SiValidation.vue";
import SiSelect from "@/atoms/SiSelect.vue";
import { LabelList } from "@/api/sdf/dal/label_list";
import { computed, ref } from "vue";
import _ from "lodash";

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

const emit = defineEmits(["update:modelValue", "error", "change"]);

const dirty = ref<boolean>(false);
const setDirty = () => {
  dirty.value = true;
  emit("change", inputValue);
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

const inputValue = computed<string | number | null>({
  get() {
    return props.modelValue;
  },
  set(value) {
    emit("update:modelValue", value);
  },
});

const boxClasses = computed((): Record<string, boolean> => {
  if (inError.value) {
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
</script>
