<template>
  <EditFormField
    :show="show"
    :validation-errors="editField.validation_errors"
    :core-edit-field="coreEditField"
  >
    <template #name>{{ editField.name }}</template>
    <template #edit>
      <input
        v-model="inputValue"
        class="pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey si-property disabled:opacity-50"
        :class="borderColor"
        type="checkbox"
        placeholder="text"
        @change="onBlur"
        @focus="onFocus"
        @blur="onBlur"
      />
    </template>
    <template #show>
      <span :class="textColor">{{ editField.value }}</span>
    </template>
  </EditFormField>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { EditField } from "@/api/sdf/dal/edit_field";
import { EditFieldService } from "@/service/edit_field";
import EditFormField from "./EditFormField.vue";
import { GlobalErrorService } from "@/service/global_error";
import { UpdateFromEditFieldResponse } from "@/service/edit_field/update_from_edit_field";
import { ApiResponse } from "@/api/sdf";

const props = defineProps<{
  show: boolean;
  coreEditField: boolean;
  editField: EditField;
}>();

const updating = ref(false);
const startValue = ref(props.editField.value);
const currentValue = ref(props.editField.value);

const onFocus = () => {
  updating.value = true;
  startValue.value = currentValue.value;
};

const onBlur = () => {
  if (currentValue.value != startValue.value) {
    EditFieldService.updateFromEditField({
      objectKind: props.editField.object_kind,
      objectId: props.editField.object_id,
      editFieldId: props.editField.id,
      value: currentValue.value,
      baggage: props.editField.baggage,
    }).subscribe((response: ApiResponse<UpdateFromEditFieldResponse>) => {
      if (response.error) {
        GlobalErrorService.set(response);
      }
    });
  }
  updating.value = false;
};

watch(
  () => props.editField,
  (editField, _prevEditField) => {
    if (!updating.value && editField.value != currentValue.value) {
      currentValue.value = editField.value;
    }
  },
);

const inputValue = computed((): boolean | undefined => {
  if (
    currentValue.value === undefined ||
    typeof currentValue.value === "boolean"
  ) {
    // We'd expect the optional value to be undefined if not provided or a
    // boolean
    return currentValue.value;
  } else if (currentValue.value === null) {
    // Or, getting a `null` implies this value has not yet been set on a
    // Component, as in "unset"
    return false;
  } else {
    // Otherwise, this is a deeply unexpected value type and we're not going to
    // let JavaScript coerce something invalid into a `bool` so, throw an error
    // instead
    throw new Error(
      `Current editField prop must be a boolean | undefined: '${JSON.stringify(
        currentValue.value,
      )}`,
    );
  }
});

const borderColor = computed(
  (): Record<string, boolean> => {
    if (props.editField.visibility_diff.kind != "None") {
      return { "input-border-gold": true };
    } else {
      return { "input-border-grey": true };
    }
  },
);
const textColor = computed(
  (): Record<string, boolean> => {
    if (props.editField.visibility_diff.kind != "None") {
      return { "text-gold": true };
    } else {
      return { "text-gold": false };
    }
  },
);
</script>

<style scoped></style>
