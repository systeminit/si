<template>
  <div class="flex flex-col w-full overflow-auto scrollbar">
    <Widgets
      :show="true"
      :edit-fields="coreEditFields"
      :core-edit-fields="true"
      :indent-level="1"
      :tree-open-state="{}"
    />
    <div
      class="pt1 pb-1 pl-6 mt-2 text-base text-white align-middle property-section-bg-color"
    >
      Properties
    </div>
    <div class="w-full">
      <Widgets
        :show="true"
        :edit-fields="propertyEditFields"
        :indent-level="1"
        :tree-open-state="treeOpenState"
        @toggle-header="toggleHeader"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";
import Widgets from "@/organisims/EditForm/Widgets.vue";
import _ from "lodash";
import {
  InitialTreeOpenStateVisitor,
  ITreeOpenState,
} from "@/utils/edit_field_visitor";

const props = defineProps<{
  editFields: EditFields;
}>();

/**
 * Returns core edit fields that are *not* component properties
 */
const coreEditFields = computed(() => {
  return _.filter(
    props.editFields,
    (field) => field.object_kind == EditFieldObjectKind.Component,
  );
});

/**
 * Returns edit fields are component properties
 */
const propertyEditFields = computed(() => {
  return _.filter(
    props.editFields,
    (field) => field.object_kind == EditFieldObjectKind.ComponentProp,
  );
});

const initialTreeOpenState = computed(
  (): ITreeOpenState => {
    const visitor = new InitialTreeOpenStateVisitor();
    visitor.visitEditFields(propertyEditFields.value);
    return visitor.initialTreeState();
  },
);

const setTreeOpenState = ref<ITreeOpenState>({});

const treeOpenState = computed(
  (): ITreeOpenState => {
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
  },
);

const toggleHeader = (fieldId: string) => {
  console.log(`toggling: ${fieldId}`, {
    setTreeOpenState: setTreeOpenState.value,
  });
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
