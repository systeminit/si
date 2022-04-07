<template>
  <template v-for="editField in props.editFields" :key="editField.id">
    <Widget
      v-if="isCoreEditField"
      :show="props.show"
      :edit-field="editField"
      :background-colors="backgroundColors"
      :core-edit-field="isCoreEditField"
      :indent-level="props.indentLevel"
      :tree-open-state="props.treeOpenState"
      :component-identification="props.componentIdentification"
      @toggle-header="toggleHeader"
    />
    <div v-else class="my-2">
      <Widget
        :show="props.show"
        :edit-field="editField"
        :background-colors="backgroundColors"
        :core-edit-field="isCoreEditField"
        :indent-level="props.indentLevel"
        :tree-open-state="props.treeOpenState"
        :component-identification="props.componentIdentification"
        @toggle-header="toggleHeader"
      />
    </div>
  </template>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { EditFields } from "@/api/sdf/dal/edit_field";
import Widget from "@/organisims/EditForm/Widget.vue";
import { interpolateColors } from "@/utils/interpolateColors";
import { ITreeOpenState } from "@/utils/edit_field_visitor";
import { ComponentIdentification } from "@/api/sdf/dal/component";

export interface WidgetsProps {
  show: boolean;
  editFields: EditFields;
  coreEditFields?: boolean;
  indentLevel: number;
  treeOpenState: ITreeOpenState;
  componentIdentification?: ComponentIdentification;
}

const props = defineProps<WidgetsProps>();

const emit = defineEmits<{
  (e: "toggleHeader", fieldId: string): void;
}>();

const toggleHeader = (fieldId: string) => {
  emit("toggleHeader", fieldId);
};

const isCoreEditField = computed(() => props.coreEditFields ?? false);

const backgroundColors = computed(() => {
  const longestProp = 50;
  return interpolateColors("rgb(50, 50, 50)", "rgb(25, 25, 25)", longestProp);
});
</script>
