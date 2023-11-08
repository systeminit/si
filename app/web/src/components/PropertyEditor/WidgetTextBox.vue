<template>
  <div v-show="isShown" class="flex content-center">
    <div class="flex grow">
      <div class="w-full">
        <SiTextBox
          :id="fieldId"
          v-model="currentValue"
          :title="props.name"
          :docLink="docLink"
          :validations="validations"
          :disabled="disabled"
          :textArea="props.textArea"
          alwaysValidate
          @blur="setField"
          @keyup.enter="triggerBlur($event)"
        />
      </div>
    </div>

    <UnsetButton
      v-if="canUnset && !disabled"
      :disabled="disableUnset"
      @click="unsetField"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, toRefs, computed, watch, onBeforeUnmount } from "vue";
import * as _ from "lodash-es";
import SiTextBox from "@/components/SiTextBox.vue";
import { usePropertyEditorIsShown } from "@/utils/usePropertyEditorIsShown";
import {
  PropertyEditorValidation,
  UpdatedProperty,
  PropertyPath,
  PropertyEditorPropKind,
} from "@/api/sdf/dal/property_editor";
import { usePropertyEditorValidations } from "@/utils/input_validations";
import UnsetButton from "./UnsetButton.vue";

const props = defineProps<{
  name: string;
  path?: PropertyPath;
  collapsedPaths: Array<Array<string>>;
  value: unknown;
  propId: string;
  valueId: string;
  propKind: PropertyEditorPropKind;
  docLink?: string;
  validation?: PropertyEditorValidation;
  disabled?: boolean;
  textArea?: boolean;
  documentation?: string;
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
    if (oldValue !== newValue) {
      if (_.isString(newValue)) {
        currentValue.value = newValue;
      } else if (_.isNumber(newValue)) {
        currentValue.value = `${newValue}`;
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
  if (
    !_.isNull(currentValue.value) &&
    currentValue.value !== props.value &&
    ((!props.value && currentValue.value) || props.value)
  ) {
    if (props.propKind === "integer") {
      emit("updatedProperty", {
        value: _.parseInt(currentValue.value, 10),
        propId: propId.value,
        valueId: valueId.value,
      });
    } else {
      emit("updatedProperty", {
        value: currentValue.value,
        propId: propId.value,
        valueId: valueId.value,
      });
    }
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

const triggerBlur = (event: KeyboardEvent) => {
  if (event?.target instanceof HTMLElement) {
    event.target.blur();
  }
};

onBeforeUnmount(setField);

const canUnset = computed(() => {
  return (props.path?.triggerPath ?? []).join(".") !== "name.si.root";
});
</script>
