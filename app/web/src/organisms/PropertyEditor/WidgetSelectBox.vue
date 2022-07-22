<template>
  <div v-show="isShown" class="flex content-center">
    <div class="flex grow">
      <div class="w-full">
        <SiSelectBox
          :id="fieldId"
          v-model="currentValue"
          :options="props.options"
          :title="props.name"
          :doc-link="docLink"
          :validations="validations"
          :disabled="disabled"
          always-validate
          @change="setField"
        />
      </div>
    </div>
    <div class="flex w-16 h-20 items-center justify-center">
      <UnsetButton
        v-if="!disabled"
        :disabled="disableUnset"
        @click="unsetField"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, toRefs, computed, watch } from "vue";
import SiSelectBox from "@/atoms/SiSelectBox.vue";
import UnsetButton from "./UnsetButton.vue";
import { LabelList } from "@/api/sdf/dal/label_list";
import { usePropertyEditorIsShown } from "@/composables/usePropertyEditorIsShown";
import {
  PropertyEditorValidation,
  UpdatedProperty,
  PropertyPath,
} from "@/api/sdf/dal/property_editor";
import _ from "lodash";
import { ValidatorArray } from "@/atoms/SiValidation.vue";

const props = defineProps<{
  name: string;
  options: LabelList<number | string>;
  path?: PropertyPath;
  collapsedPaths: Array<Array<string>>;
  value: unknown;
  propId: number;
  valueId: number;
  docLink?: string;
  validation?: PropertyEditorValidation;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: "updatedProperty", v: UpdatedProperty): void;
}>();

const { name, path, collapsedPaths, valueId, propId, value, validation } =
  toRefs(props);

const currentValue = ref<string | number | null>(null);
watch(
  value,
  (newValue) => {
    if (currentValue.value !== newValue) {
      currentValue.value = newValue as number | string | null;
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
  if (currentValue.value != (props.value ?? undefined)) {
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

const validations = computed(() => {
  const results: ValidatorArray = [];
  if (validation?.value) {
    for (let x = 0; x < validation.value.errors.length; x++) {
      const error = validation.value.errors[x];
      results.push({
        id: `${x}`,
        message: error.message,
        check: () => false,
      });
    }
  }
  return results;
});
</script>
