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
    v-else-if="props.editField.widget.kind === 'Array' && attributeContext"
    :show="props.show"
    :edit-field="props.editField"
    :core-edit-field="props.coreEditField"
    :indent-level="props.indentLevel"
    :tree-open-state="props.treeOpenState"
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
  <div v-else>
    Error: could not create AttributeContext for widget kind
    {{ props.editField.widget.kind }}
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
import { ComponentIdentification } from "@/api/sdf/dal/component";
import { computed } from "vue";
import { AttributeContext } from "@/api/sdf/dal/attribute";

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
const attributeContext = computed((): AttributeContext | "" => {
  if (!props.editField.baggage || !props.componentIdentification) {
    return "";
  }
  return {
    attribute_context_prop_id: props.editField.baggage.prop_id,
    attribute_context_schema_id: props.componentIdentification.schemaId,
    attribute_context_schema_variant_id:
      props.componentIdentification.schemaVariantId,
    attribute_context_component_id: props.componentIdentification.componentId,
    attribute_context_system_id: -1,
  };
});

const emit = defineEmits<{
  (e: "toggleHeader", fieldId: string): void;
}>();

const toggleHeader = (fieldId: string) => {
  emit("toggleHeader", fieldId);
};
</script>
