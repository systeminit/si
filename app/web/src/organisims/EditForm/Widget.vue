<template>
  <HeaderWidget
    v-if="editField.widget.kind === 'Header'"
    :show="true"
    :edit-field="editField"
    :background-colors="backgroundColors"
    :core-edit-field="coreEditField"
    :indent-level="indentLevel"
  />
  <ArrayWidget
    v-else-if="editField.widget.kind === 'Array'"
    :show="true"
    :edit-field="editField"
    :core-edit-field="coreEditField"
    :indent-level="indentLevel"
  />
  <TextWidget
    v-else-if="editField.widget.kind === 'Text'"
    :show="true"
    :edit-field="editField"
    :core-edit-field="coreEditField"
  />
  <CheckboxWidget
    v-else-if="editField.widget.kind === 'Checkbox'"
    :show="true"
    :edit-field="editField"
    :core-edit-field="coreEditField"
  />
  <SelectWidget
    v-else-if="editField.widget.kind === 'Select'"
    :show="true"
    :edit-field="editField"
    :core-edit-field="coreEditField"
  />
</template>

<script setup lang="ts">
import { EditField } from "@/api/sdf/dal/edit_field";
import CheckboxWidget from "@/organisims/EditForm/CheckboxWidget.vue";
import TextWidget from "@/organisims/EditForm/TextWidget.vue";
import SelectWidget from "@/organisims/EditForm/SelectWidget.vue";
import HeaderWidget from "@/organisims/EditForm/HeaderWidget.vue";
import ArrayWidget from "@/organisims/EditForm/ArrayWidget.vue";

defineProps<{
  coreEditField: boolean;
  indentLevel: number;
  editField: EditField;
  backgroundColors: number[][];
}>();
</script>
