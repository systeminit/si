<template>
  <HeaderWidget
    v-if="props.editField.widget.kind === 'Header'"
    :show="props.show"
    :edit-field="props.editField"
    :background-colors="props.backgroundColors"
    :core-edit-field="props.coreEditField"
    :indent-level="props.indentLevel"
    :tree-open-state="props.treeOpenState"
    :component-identification="props.componentIdentification"
    @toggle-header="toggleHeader"
  />
  <ArrayWidget
    v-else-if="props.editField.widget.kind === 'Array' && attributeContext"
    :show="props.show"
    :edit-field="props.editField"
    :core-edit-field="props.coreEditField"
    :indent-level="props.indentLevel"
    :tree-open-state="props.treeOpenState"
    :component-identification="props.componentIdentification"
    :attribute-context="attributeContext"
  />
  <MapWidget
    v-else-if="props.editField.widget.kind === 'Map' && attributeContext"
    :show="props.show"
    :edit-field="props.editField"
    :core-edit-field="props.coreEditField"
    :indent-level="props.indentLevel"
    :tree-open-state="props.treeOpenState"
    :component-identification="props.componentIdentification"
    :attribute-context="attributeContext"
  />
  <TextWidget
    v-else-if="props.editField.widget.kind === 'Text' && attributeContext"
    :show="props.show"
    :edit-field="props.editField"
    :core-edit-field="props.coreEditField"
    :attribute-context="attributeContext"
  />
  <CheckboxWidget
    v-else-if="props.editField.widget.kind === 'Checkbox' && attributeContext"
    :show="props.show"
    :edit-field="props.editField"
    :core-edit-field="props.coreEditField"
    :attribute-context="attributeContext"
  />
  <SelectWidget
    v-else-if="props.editField.widget.kind === 'Select' && attributeContext"
    :show="props.show"
    :edit-field="props.editField"
    :core-edit-field="props.coreEditField"
    :attribute-context="attributeContext"
  />
  <div v-else class="text-xs text-red-400">
    Error: WidgetKind not found or could not create AttributeContext for
    WidgetKind ({{ props.editField.widget.kind }})
  </div>
</template>

<script setup lang="ts">
import { EditField } from "@/api/sdf/dal/edit_field";
import CheckboxWidget from "@/organisims/EditForm/CheckboxWidget.vue";
import TextWidget from "@/organisims/EditForm/TextWidget.vue";
import SelectWidget from "@/organisims/EditForm/SelectWidget.vue";
import HeaderWidget from "@/organisims/EditForm/HeaderWidget.vue";
import ArrayWidget from "@/organisims/EditForm/ArrayWidget.vue";
import { ITreeOpenState } from "@/utils/edit_field_visitor";
import { buildAttributeContext } from "@/utils/attributeContext";
import { ComponentIdentification } from "@/api/sdf/dal/component";
import { computed } from "vue";
import { AttributeContext } from "@/api/sdf/dal/attribute";
import MapWidget from "@/organisims/EditForm/MapWidget.vue";

const props = defineProps<{
  show: boolean;
  coreEditField: boolean;
  indentLevel: number;
  editField: EditField;
  treeOpenState: ITreeOpenState;
  backgroundColors: number[][];
  componentIdentification?: ComponentIdentification;
}>();

// FIXME(nick): handle SystemId.
const attributeContext = computed((): AttributeContext | undefined => {
  // NOTE: these widgets are used in both the attritbute editor and the schema editor. At the
  // moment, the schema editor may not have baggage which may change in the future. If baggage
  // becomes required everywhere for all edit fields, then this check should be removed.
  if (!props.editField.baggage) {
    return undefined;
  }
  return buildAttributeContext(
    props.editField.baggage,
    props.componentIdentification,
  );
});

const emit = defineEmits<{
  (e: "toggleHeader", fieldId: string): void;
}>();

const toggleHeader = (fieldId: string) => {
  emit("toggleHeader", fieldId);
};
</script>
