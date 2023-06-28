<template>
  <div v-show="isShown" class="flex content-center w-full">
    <div class="flex grow">
      <div class="w-full">
        <SiSelectBox
          :id="fieldId"
          v-model="currentValue"
          :options="props.options"
          :title="props.name"
          :docLink="docLink"
          :validations="validations"
          :disabled="disabled"
          alwaysValidate
          @change="setField"
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
import { ref, toRefs, computed, watch } from "vue";
import * as _ from "lodash-es";
import SiSelectBox from "@/components/SiSelectBox.vue";
import { LabelList } from "@/api/sdf/dal/label_list";
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
  options: LabelList<number | string>;
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
