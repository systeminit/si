<template>
  <!-- NOTE(nick): "EditFormField" is the same as "Field" from demo v0.4 -->
  <EditFormField
    :show="show"
    :core-edit-field="coreEditField"
    :validation-errors="editField.validation_errors"
  >
    <template #name>
      {{ editField.name }}
    </template>
    <template #edit>
      <input
        v-model="currentValue"
        class="pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey si-property disabled:opacity-50"
        :class="inputStyles"
        type="text"
        aria-label="name"
        placeholder="text"
        @keyup.enter="onKeyEnter($event)"
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
import EditFormField from "./EditFormField.vue";
import { EditFieldService } from "@/service/edit_field";
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
const onKeyEnter = (event: KeyboardEvent) => {
  if (event?.target instanceof HTMLElement) {
    event.target.blur();
  }
};
watch(
  () => props.editField,
  (editField, _prevEditField) => {
    if (!updating.value && editField.value != currentValue.value) {
      currentValue.value = editField.value;
    }
  },
);

const inputStyles = computed(
  (): Record<string, boolean> => {
    let styles: Record<string, boolean> = {};

    if (props.editField.visibility_diff.kind != "None") {
      styles["input-border-gold"] = true;
      styles["input-border-grey"] = false;
    } else {
      styles["input-border-gold"] = false;
      styles["input-border-grey"] = true;
    }
    if (props.coreEditField) {
      styles["flex-grow"] = true;
    } else {
      styles["flex-grow"] = false;
    }

    return styles;
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
