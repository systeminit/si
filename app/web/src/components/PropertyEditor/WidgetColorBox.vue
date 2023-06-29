<template>
  <div v-show="isShown" class="flex content-center">
    <div class="flex grow">
      <div class="w-full">
        <SiColorBox
          :id="fieldId"
          v-model="currentValue"
          :title="props.name"
          :docLink="docLink"
          :validations="validations"
          :disabled="disabled"
          alwaysValidate
          @blur="setField"
          @keyup.enter="triggerBlur($event)"
        />
      </div>
    </div>

    <UnsetButton
      v-if="!disabled"
      :disabled="disableUnset"
      @click="unsetField"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, toRefs, computed, watch, onBeforeUnmount } from "vue";
import * as _ from "lodash-es";
import SiColorBox from "@/components/SiColorBox.vue";
import { usePropertyEditorIsShown } from "@/utils/usePropertyEditorIsShown";
import {
  PropertyEditorValidation,
  UpdatedProperty,
  PropertyPath,
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

const triggerBlur = (event: KeyboardEvent) => {
  if (event?.target instanceof HTMLElement) {
    event.target.blur();
  }
};

onBeforeUnmount(setField);
</script>
