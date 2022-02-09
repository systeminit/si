<template>
  <EditFormField :show="show" :validation-errors="editField.validation_errors">
    <template #name>
      {{ editField.name }}
    </template>
    <template #edit>
      <div class="flex flex-col mt-1">
        <div
          v-for="(editFields, index) in widget.options.entries"
          :key="index"
          class="flex flex-col justify-between w-full p-1 border border-gray-500"
        >
          <Widgets :edit-fields="editFields" />
        </div>
        <div class="flex flex-row mt-1 ml-1">
          <button @click="addToArray">
            <VueFeather type="plus" />
          </button>
        </div>
      </div>
    </template>
    <template #show>
      <div class="flex flex-col">
        <div
          v-for="(editFields, index) in widget.options.entries"
          :key="index"
          class="flex flex-col justify-between w-full mx-1 border border-gray-500"
        >
          <Widgets :edit-fields="editFields" />
        </div>
      </div>
    </template>
  </EditFormField>
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import type { EditField } from "@/api/sdf/dal/edit_field";
import EditFormField from "./EditFormField.vue";
import { ArrayWidgetDal } from "@/api/sdf/dal/edit_field";
import VueFeather from "vue-feather";
import { EditFieldService } from "@/service/edit_field";
import { ApiResponse } from "@/api/sdf";
import { UpdateFromEditFieldResponse } from "@/service/edit_field/update_from_edit_field";
import { GlobalErrorService } from "@/service/global_error";
import { defineAsyncComponent, DefineComponent } from "vue";
import type { WidgetsProps } from "./Widgets.vue";

// Eliminate the circular dependency of HeaderWidget -> Widgets -> HeaderWidget
// by using `defineAsyncComponent` in a careful way to preserve the ability for
// typeechecking to work with `tsc` and the `volar` language server used in
// VSCode/NeoVim/Vim.
//
// See:
// https://github.com/johnsoncodehk/volar/issues/644#issuecomment-1012716529
const Widgets = defineAsyncComponent<DefineComponent<WidgetsProps>>(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  () => import("./Widgets.vue") as any,
);

const props = defineProps({
  show: {
    type: Boolean,
    required: true,
  },
  editField: {
    type: Object as PropType<EditField>,
    required: true,
  },
});

const widget = computed<ArrayWidgetDal>(() => {
  return props.editField.widget as ArrayWidgetDal;
});

const addToArray = () => {
  EditFieldService.updateFromEditField({
    objectKind: props.editField.object_kind,
    objectId: props.editField.object_id,
    editFieldId: props.editField.id,
    value: null,
    baggage: props.editField.baggage,
  }).subscribe((response: ApiResponse<UpdateFromEditFieldResponse>) => {
    if (response.error) {
      GlobalErrorService.set(response);
    }
  });
};
</script>

<style scoped></style>
