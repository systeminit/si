<template>
  <div v-show="isShown" class="flex flex-col content-center w-full">
    <label
      :for="fieldId"
      class="block text-sm font-medium text-neutral-900 dark:text-neutral-50 whitespace-nowrap"
    >
      {{ props.name }} <span v-if="required">(required)</span>
    </label>

    <div class="flex items-center">
      <SiComboBox
        :id="fieldId"
        v-model="currentValue"
        class="flex-grow"
        :options="props.options"
        :title="props.name"
        :doc-link="docLink"
        :disabled="disabled"
        :validations="validations"
        always-validate
        @change="setField"
      />
      <UnsetButton
        v-if="!disabled"
        class="mt-0"
        :disabled="disableUnset"
        @click="unsetField"
      />
    </div>

    <p v-if="docLink" class="mt-2 text-xs text-action-500">
      <a :href="docLink" target="_blank" class="hover:underline">
        Documentation
      </a>
    </p>

    <p v-if="description" class="mt-2 text-xs text-neutral-300">
      {{ description }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, toRefs, computed, watch, toRef } from "vue";
import SiComboBox from "@/atoms/SiComboBox.vue";
import { LabelList } from "@/api/sdf/dal/label_list";
import { usePropertyEditorIsShown } from "@/composables/usePropertyEditorIsShown";
import {
  UpdatedProperty,
  PropertyPath,
  PropertyEditorValidation,
} from "@/api/sdf/dal/property_editor";
import { usePropertyEditorValidations } from "@/utils/input_validations";
import UnsetButton from "./UnsetButton.vue";

const props = defineProps<{
  name: string;
  options: LabelList<number | string>;
  path?: PropertyPath;
  collapsedPaths: Array<Array<string>>;
  value: unknown;
  propId: number;
  valueId: number;
  validation?: PropertyEditorValidation;
  docLink?: string;
  disabled?: boolean;
  required?: boolean; // NOTE(victor) this was being passed down as undefined. Keeping it since we'll use it someday.
  description?: string; // NOTE(victor) this was being passed down as undefined. Keeping it since we'll use it someday.
}>();

const validation = toRef(props, "validation", undefined);

const emit = defineEmits<{
  (e: "updatedProperty", v: UpdatedProperty): void;
}>();

const { name, path, collapsedPaths, valueId, propId, value } = toRefs(props);

const currentValue = ref<string | number | undefined>(undefined);
watch(
  value,
  (newValue) => {
    if (currentValue.value !== newValue) {
      currentValue.value = newValue as number | string | undefined;
    }
  },
  { immediate: true },
);

const disableUnset = computed(() => {
  if ((value.value ?? null) === null) {
    return true;
  } else {
    return false;
  }
});

// @ts-ignore
const fieldId = ref(props.path.triggerPath.join("."));

const { isShown } = usePropertyEditorIsShown(name, collapsedPaths, path);

const setField = () => {
  if (currentValue.value !== (props.value ?? undefined)) {
    emit("updatedProperty", {
      value: currentValue.value,
      propId: propId.value,
      valueId: valueId.value,
    });
  }
};

const unsetField = () => {
  emit("updatedProperty", {
    value: null,
    propId: propId.value,
    valueId: valueId.value,
  });
};

const validations = usePropertyEditorValidations(validation);
</script>
