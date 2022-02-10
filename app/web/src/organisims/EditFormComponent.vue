<template>
  <div class="flex flex-col w-full overflow-auto scrollbar">
    <Widgets
      v-if="coreEditFields"
      :edit-fields="coreEditFields"
      :core-edit-fields="true"
      :indent-level="1"
    />
    <div
      v-if="propertyEditFields"
      class="pt1 pb-1 pl-6 mt-2 text-base text-white align-middle property-section-bg-color"
    >
      Properties
    </div>
    <div class="w-full">
      <Widgets
        v-if="propertyEditFields"
        :edit-fields="propertyEditFields"
        :indent-level="1"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";
import { fromRef, refFrom } from "vuse-rx";
import { EditFieldService } from "@/service/edit_field";
import { GlobalErrorService } from "@/service/global_error";
import Widgets from "@/organisims/EditForm/Widgets.vue";
import { combineLatest, from } from "rxjs";
import { switchMap } from "rxjs/operators";
import _ from "lodash";

const props = defineProps<{
  objectId: number;
}>();

const props$ = fromRef(props, { immediate: true, deep: true });

/**
 * Returns core edit fields that are *not* component properties
 */
const coreEditFields = computed(() => {
  if (editFields.value === undefined) {
    return undefined;
  } else {
    return _.filter(
      editFields.value,
      (field) => field.object_kind == EditFieldObjectKind.Component,
    );
  }
});

/**
 * Returns edit fields are component properties
 */
const propertyEditFields = computed(() => {
  if (editFields.value === undefined) {
    return undefined;
  } else {
    return _.filter(
      editFields.value,
      (field) => field.object_kind == EditFieldObjectKind.ComponentProp,
    );
  }
});

const editFields = refFrom<EditFields | undefined>(
  combineLatest([props$]).pipe(
    switchMap(([props]) => {
      return EditFieldService.getEditFields({
        id: props.objectId,
        objectKind: EditFieldObjectKind.Component,
      });
    }),
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([[]]);
      } else {
        return from([response.fields]);
      }
    }),
  ),
);
</script>

<style scoped>
.property-section-bg-color {
  background-color: #292c2d;
}
</style>
