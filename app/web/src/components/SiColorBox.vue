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
      <ColorPicker :id="id" v-model="color" @change="updateValue" />

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
      :value="color"
      :validations="props.validations"
      class="mt-2"
      @errors="setInError($event)"
    />
  </div>
</template>

<script setup lang="ts">
import { PropType, toRefs, ref } from "vue";
import * as _ from "lodash-es";
import { Icon } from "@si/vue-lib/design-system";
import { useFormSettings } from "@/utils/formSettings";
import { ValidatorArray, useValidations } from "@/utils/input_validations";
import SiValidation from "./SiValidation.vue";
import ColorPicker from "./ColorPicker.vue";

const props = defineProps({
  modelValue: { type: String, required: true },
  title: String,
  id: { type: String },
  description: String,

  validations: { type: Array as PropType<ValidatorArray> },
  required: Boolean,
  alwaysValidate: Boolean,

  docLink: String,

  disabled: Boolean,
});

const { alwaysValidate } = toRefs(props);

const formSettings = useFormSettings();

const emit = defineEmits(["update:modelValue", "error", "blur"]);

const color = ref(props.modelValue ?? "#AABBCC");

const { inError, setInError, setDirty } = useValidations(
  alwaysValidate,
  () => emit("blur", color),
  (inError: boolean) => emit("error", inError),
);

const updateValue = (color: string) => {
  setDirty();
  emit("update:modelValue", color);
  emit("blur");
};
</script>

<script lang="ts">
export default {
  inheritAttrs: false,
};
</script>
