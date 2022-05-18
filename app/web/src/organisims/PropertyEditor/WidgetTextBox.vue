<template>
  <div v-show="isShown" class="flex content-center">
    <div class="flex grow">
      <div class="w-full">
        <SiTextBox
          :id="fieldId"
          v-model="currentValue"
          :title="props.name"
          :doc-link="docLink"
          :validations="validations"
          :disabled="disabled"
          always-validate
          @blur="setField"
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
import SiTextBox from "@/atoms/SiTextBox2.vue";
import UnsetButton from "./UnsetButton.vue";
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

const currentValue = ref<string>("");
watch(
  value,
  (newValue, oldValue) => {
    if (oldValue != newValue) {
      if (_.isString(newValue)) {
        currentValue.value = newValue;
      } else {
        currentValue.value = "";
      }
    }
  },
  { immediate: true },
);

const disableUnset = computed(() => {
  if (_.isNull(value.value)) {
    return true;
  } else {
    return false;
  }
});

// @ts-ignore
const fieldId = ref(props.path.triggerPath.join("."));

const { isShown } = usePropertyEditorIsShown(name, collapsedPaths, path);

const setField = () => {
  console.log("set field", {
    fv: currentValue.value,
    propId: propId.value,
    valueId: valueId.value,
  });
  if (!_.isNull(currentValue.value)) {
    emit("updatedProperty", {
      value: currentValue.value,
      propId: propId.value,
      valueId: valueId.value,
    });
  }
};

const unsetField = () => {
  console.log("unset field", {
    fv: currentValue.value,
    propId: propId.value,
    valueId: valueId.value,
  });
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
