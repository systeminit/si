<template>
  <div class="flex flex-row w-full" v-for="editField in editFields" :key="editField.id">
    <!-- eventually this will do the show/hide logic -->
    <HeaderWidget v-if="editField.widget.kind === 'Header'" :show="true" :edit-field="editField" />
    <ArrayWidget
      v-else-if="editField.widget.kind === 'Array'"
      :show="true"
      :edit-field="editField"
    />
    <TextWidget v-else-if="editField.widget.kind === 'Text'" :show="true" :edit-field="editField" />
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
import {PropType, ref} from "vue";
import {EditFields} from "@/api/sdf/dal/edit_field";
import CheckboxWidget from "@/organisims/EditForm/CheckboxWidget.vue";
import TextWidget from "@/organisims/EditForm/TextWidget.vue";
import SelectWidget from "@/organisims/EditForm/SelectWidget.vue";
import HeaderWidget from "@/organisims/EditForm/HeaderWidget.vue";
import ArrayWidget from "@/organisims/EditForm/ArrayWidget.vue";

const props = defineProps({
  editFields: {
    type: Array as PropType<EditFields>,
    required: true,
  },
});

type TreeOpenState = Record<string, boolean>;
const treeOpenState = ref<TreeOpenState>({});
</script>
