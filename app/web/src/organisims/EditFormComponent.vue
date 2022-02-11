<template>
  <div class="flex flex-col w-full overflow-auto scrollbar">
    <Widgets
      :edit-fields="coreEditFields"
      :core-edit-fields="true"
      :indent-level="1"
    />
    <div
      class="pt1 pb-1 pl-6 mt-2 text-base text-white align-middle property-section-bg-color"
    >
      Properties
    </div>
    <div class="w-full">
      <Widgets :edit-fields="propertyEditFields" :indent-level="1" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";
import Widgets from "@/organisims/EditForm/Widgets.vue";
import _ from "lodash";

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
</script>

<style scoped>
.property-section-bg-color {
  background-color: #292c2d;
}
</style>
