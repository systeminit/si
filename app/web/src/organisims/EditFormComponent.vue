<template>
  <div class="flex flex-col w-full overflow-auto scrollbar">
    <Widgets
      :show="true"
      :edit-fields="coreEditFields"
      :core-edit-fields="true"
      :indent-level="1"
      :tree-open-state="{}"
      :component-identification="componentIdentification"
    />
    <div
      class="pt-1 pb-1 pl-6 mt-2 text-base text-white align-middle property-section-bg-color"
    >
      Properties
    </div>
    <div class="flex flex-col w-full overflow-auto scrollbar">
      <Widgets
        :show="true"
        :edit-fields="propertyEditFields"
        :indent-level="1"
        :tree-open-state="treeOpenState"
        :component-identification="componentIdentification"
        @toggle-header="toggleHeader"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import type {
  EditFields,
  EditField,
  HeaderWidgetDal,
} from "@/api/sdf/dal/edit_field";
import { EditFieldDataType } from "@/api/sdf/dal/edit_field";
import Widgets from "@/organisims/EditForm/Widgets.vue";
import {
  InitialTreeOpenStateVisitor,
  ITreeOpenState,
} from "@/utils/edit_field_visitor";
import { ComponentIdentification } from "@/api/sdf/dal/component";

const props = defineProps<{
  editFields: EditFields;
  componentIdentification: ComponentIdentification;
}>();

/**
 * Returns core edit fields that are *not* component properties
 */
const coreEditFields = computed(() => {
  let fields: Array<EditField> = [];
  props.editFields.forEach((root) => {
    if (root.data_type === EditFieldDataType.Object) {
      const widget = root.widget as HeaderWidgetDal;
      widget.options.edit_fields
        .filter((p) => p.name === "si")
        .forEach((p) => {
          if (p.data_type === EditFieldDataType.Object) {
            const w = p.widget as HeaderWidgetDal;
            fields = fields.concat(w.options.edit_fields);
          }
        });
    }
  });
  return fields;
});

/**
 * Returns edit fields are component properties
 */
const propertyEditFields = computed(() => {
  let fields: Array<EditField> = [];
  props.editFields.forEach((root) => {
    if (root.data_type === EditFieldDataType.Object) {
      const widget = root.widget as HeaderWidgetDal;
      widget.options.edit_fields
        .filter((p) => p.name === "domain")
        .forEach((p) => {
          if (p.data_type === EditFieldDataType.Object) {
            const w = p.widget as HeaderWidgetDal;
            fields = fields.concat(w.options.edit_fields);
          }
        });
    }
  });
  return fields;
});

const initialTreeOpenState = computed((): ITreeOpenState => {
  const visitor = new InitialTreeOpenStateVisitor();
  visitor.visitEditFields(propertyEditFields.value);
  return visitor.initialTreeState();
});

const setTreeOpenState = ref<ITreeOpenState>({});

const treeOpenState = computed((): ITreeOpenState => {
  const state: ITreeOpenState = {};
  for (const [fieldId, initialOpenState] of Object.entries(
    initialTreeOpenState.value,
  )) {
    const setOpenState = setTreeOpenState.value[fieldId];
    if (setOpenState === undefined) {
      state[fieldId] = initialOpenState;
    } else {
      state[fieldId] = setOpenState;
    }
  }
  return state;
});

const toggleHeader = (fieldId: string) => {
  if (setTreeOpenState.value[fieldId] === undefined) {
    setTreeOpenState.value[fieldId] = true;
  } else {
    setTreeOpenState.value[fieldId] = !setTreeOpenState.value[fieldId];
  }
};
</script>

<style scoped>
.property-section-bg-color {
  background-color: #292c2d;
}
</style>
