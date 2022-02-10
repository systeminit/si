<template>
  <template v-for="editField in editFields" :key="editField.id">
    <Widget
      v-if="isCoreEditField"
      :show="true"
      :edit-field="editField"
      :background-colors="backgroundColors"
      :core-edit-field="isCoreEditField"
      :indent-level="indentLevel"
    />
    <div v-else class="my-2">
      <Widget
        :show="true"
        :edit-field="editField"
        :background-colors="backgroundColors"
        :core-edit-field="isCoreEditField"
        :indent-level="indentLevel"
      />
    </div>
  </template>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { EditFields } from "@/api/sdf/dal/edit_field";
import Widget from "@/organisims/EditForm/Widget.vue";
import { interpolateColors } from "@/utils/interpolateColors";

export interface WidgetsProps {
  editFields: EditFields;
  coreEditFields?: boolean;
  indentLevel: number;
}
const props = defineProps<WidgetsProps>();

const isCoreEditField = computed(() =>
  props.coreEditFields === undefined ? false : props.coreEditFields,
);

const backgroundColors = computed(() => {
  const longestProp = 50;
  // for (const field of props.editFields) {
  //   console.log("field", { field });
  // }

  const colors = interpolateColors(
    "rgb(50, 50, 50)",
    "rgb(25, 25, 25)",
    longestProp,
  );

  return colors;
});
</script>
