<template>
  <div>
    <label v-if="props.title" :for="props.id" class="block text-sm font-medium">
      {{ title }}
      <span
        v-if="props.required && !formSettings.hideRequiredLabel"
        :class="formSettings.requiredLabelClasses"
        >{{ formSettings.requiredLabel }}</span
      >
    </label>

    <div class="mt-1 relative">
      <span class="flex">
        <span
          :id="props.id"
          ref="pickerElement"
          :aria-required="props.required"
          :style="{ backgroundColor: props.modelValue }"
          class="block w-10 h-6 py-2 border rounded-sm shadow-sm focus:outline-none sm:text-sm dark:color-white"
          :class="boxClasses"
        ></span>
        <span class="p-1">{{ props.modelValue }}</span>
      </span>

      <div
        v-if="inError"
        class="absolute inset-y-0 right-0 pr-2 flex items-center text-destructive-400"
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
      {{ description }}
    </p>

    <SiValidation
      :value="String(inputValue)"
      :validations="props.validations"
      class="mt-2"
      @errors="setInError($event)"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, PropType, toRefs, onMounted, ref } from "vue";
import _ from "lodash";
import Picker from "vanilla-picker";
import { useFormSettings } from "@/utils/formSettings";
import { ValidatorArray, useValidations } from "@/utils/input_validations";
import Icon from "@/ui-lib/icons/Icon.vue";
import SiValidation from "./SiValidation.vue";

const props = defineProps({
  modelValue: { type: String },
  title: String,
  id: { type: String, required: true },
  description: String,

  validations: { type: Array as PropType<ValidatorArray> },
  required: Boolean,
  alwaysValidate: Boolean,

  docLink: String,

  disabled: Boolean,
});

const pickerElement = ref<HTMLElement | null>(null);
const picker = ref<Picker | null>(null);
onMounted(() => {
  if (!props.disabled) {
    const p = new Picker(pickerElement.value as HTMLElement);
    p.onDone = valueChanged;
    picker.value = p;
  }
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

const boxClasses = computed((): Record<string, boolean> => {
  const results: Record<string, boolean> = {};
  if (!props.disabled) {
    results["cursor-pointer"] = true;
  }
  return results;
});

const valueChanged = (color: { hex: string }) => {
  setDirty();
  emit("update:modelValue", color.hex);
  emit("blur");
};
</script>

<script lang="ts">
export default {
  inheritAttrs: false,
};
</script>

<style>
.picker_wrapper.popup,
.picker_wrapper.popup .picker_arrow::before,
.picker_wrapper.popup .picker_arrow::after {
  background: unset;
}

.picker_wrapper,
.picker_editor,
.picker_editor input::placeholder {
  color: inherit !important;
}
</style>
