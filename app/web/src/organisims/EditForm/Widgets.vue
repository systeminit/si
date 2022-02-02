<template>
  <div v-for="editField in editFields" :key="editField.id" class="my-2">
    <!-- eventually this will do the show/hide logic -->
    <HeaderWidget
      v-if="editField.widget.kind === 'Header'"
      :show="true"
      :edit-field="editField"
      :background-colors="backgroundColors"
    />
    <ArrayWidget
      v-else-if="editField.widget.kind === 'Array'"
      :show="true"
      :edit-field="editField"
    />
    <TextWidget
      v-else-if="editField.widget.kind === 'Text'"
      :show="true"
      :edit-field="editField"
    />
    <CheckboxWidget
      v-else-if="editField.widget.kind === 'Checkbox'"
      :show="true"
      :edit-field="editField"
    />
    <SelectWidget
      v-else-if="editField.widget.kind === 'Select'"
      :show="true"
      :edit-field="editField"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import { EditFields } from "@/api/sdf/dal/edit_field";
import CheckboxWidget from "@/organisims/EditForm/CheckboxWidget.vue";
import TextWidget from "@/organisims/EditForm/TextWidget.vue";
import SelectWidget from "@/organisims/EditForm/SelectWidget.vue";
import HeaderWidget from "@/organisims/EditForm/HeaderWidget.vue";
import ArrayWidget from "@/organisims/EditForm/ArrayWidget.vue";
import { interpolateColors } from "@/utils/interpolateColors";

export interface WidgetsProps {
  editFields: EditFields;
}

const props = defineProps({
  editFields: {
    type: Array as PropType<EditFields>,
    required: true,
  },
});

const backgroundColors = computed(() => {
  const longestProp = 50;
  for (const field of props.editFields) {
    console.log("field", { field });
  }

  const colors = interpolateColors(
    "rgb(50, 50, 50)",
    "rgb(25, 25, 25)",
    longestProp,
  );

  return colors;
});

// type TreeOpenState = Record<string, boolean>;
// const treeOpenState = ref<TreeOpenState>({});
</script>
