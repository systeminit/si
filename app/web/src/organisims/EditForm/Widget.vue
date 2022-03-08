<template>
  <HeaderWidget
    v-if="props.editField.widget.kind === 'Header'"
    :show="props.show"
    :edit-field="props.editField"
    :background-colors="props.backgroundColors"
    :core-edit-field="props.coreEditField"
    :indent-level="props.indentLevel"
    :tree-open-state="props.treeOpenState"
    @toggle-header="toggleHeader"
  />
  <ArrayWidget
    v-else-if="props.editField.widget.kind === 'Array'"
    :show="props.show"
    :edit-field="props.editField"
    :core-edit-field="props.coreEditField"
    :indent-level="props.indentLevel"
    :tree-open-state="props.treeOpenState"
  />
  <TextWidget
    v-else-if="props.editField.widget.kind === 'Text'"
    :show="props.show"
    :edit-field="props.editField"
    :core-edit-field="props.coreEditField"
  />
  <CheckboxWidget
    v-else-if="props.editField.widget.kind === 'Checkbox'"
    :show="props.show"
    :edit-field="props.editField"
    :core-edit-field="props.coreEditField"
  />
  <SelectWidget
    v-else-if="props.editField.widget.kind === 'Select'"
    :show="props.show"
    :edit-field="props.editField"
    :core-edit-field="props.coreEditField"
  />
</template>

<script setup lang="ts">
import { EditField } from "@/api/sdf/dal/edit_field";
import CheckboxWidget from "@/organisims/EditForm/CheckboxWidget.vue";
import TextWidget from "@/organisims/EditForm/TextWidget.vue";
import SelectWidget from "@/organisims/EditForm/SelectWidget.vue";
import HeaderWidget from "@/organisims/EditForm/HeaderWidget.vue";
import ArrayWidget from "@/organisims/EditForm/ArrayWidget.vue";
import { ITreeOpenState } from "@/utils/edit_field_visitor";

const props = defineProps<{
  show: boolean;
  coreEditField: boolean;
  indentLevel: number;
  editField: EditField;
  treeOpenState: ITreeOpenState;
  backgroundColors: number[][];
}>();

const emit = defineEmits<{
  (e: "toggleHeader", fieldId: string): void;
}>();

const toggleHeader = (fieldId: string) => {
  emit("toggleHeader", fieldId);
};
</script>
