<template>
  <div v-show="isShown" class="flex content-center">
    <div class="w-full">
      <SiCheckBox
        :id="fieldId"
        v-model="currentValue"
        :title="props.name"
        :docLink="docLink"
        :validations="validations"
        :disabled="disabled"
        alwaysValidate
        @blur="setField"
      />
    </div>
    <UnsetButton
      v-if="!disabled"
      :disabled="disableUnset"
      class="-mt-0.5"
      @click="unsetField"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, toRefs, computed, watch } from "vue";
import * as _ from "lodash-es";
import SiCheckBox from "@/components/SiCheckBox.vue";
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

const currentValue = ref<boolean | undefined>(undefined);
watch(
  value,
  (newValue, oldValue) => {
    if (oldValue !== newValue) {
      if (_.isBoolean(newValue)) {
        currentValue.value = newValue;
      } else {
        currentValue.value = undefined;
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
  if (!_.isNull(currentValue.value) && currentValue.value !== props.value) {
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
