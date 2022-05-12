<template>
  <EditFormField
    :show="show"
    :validation-errors="props.editField.validation_errors"
    :core-edit-field="coreEditField"
  >
    <template #name>
      <SiLink
        v-if="props.editField.baggage?.prop_doc_link"
        :uri="props.editField.baggage.prop_doc_link"
        :blank-target="true"
        class="flex flex-row justify-end"
      >
        <span class="flex flex-col content-center justify-center">
          {{ props.editField.name }}
        </span>
        <VueFeather type="help-circle" size="1em" class="m-2" />
      </SiLink>
      <template v-else>
        {{ props.editField.name }}
      </template>
    </template>

    <template #edit>
      <div class="flex flex-col mt-1">
        <div
          v-if="widget.options.entries"
          class="flex flex-col justify-between w-full mx-1 border border-gray-500"
        >
          <Widgets
            :show="show"
            :edit-fields="widget.options.entries"
            :indent-level="indentLevel + 1"
            :component-identification="props.componentIdentification"
            :tree-open-state="treeOpenState"
          />
        </div>
        <div class="flex flex-row mt-1 ml-1">
          <button @click="addToArray">
            <VueFeather type="plus" />
          </button>
        </div>
        <div
          v-if="!coreEditField"
          class="flex flex-row items-center w-10 ml-1 bg-red"
        >
          <Unset
            :edit-field="props.editField"
            :attribute-context="props.attributeContext"
          />
        </div>
      </div>
    </template>
    <template #show>
      <div class="flex flex-col">
        <div
          v-if="widget.options.entries"
          class="flex flex-col justify-between w-full mx-1 border border-gray-500"
        >
          <Widgets
            :show="show"
            :edit-fields="widget.options.entries"
            :indent-level="indentLevel + 1"
            :tree-open-state="treeOpenState"
          />
        </div>
      </div>
    </template>
  </EditFormField>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { EditField } from "@/api/sdf/dal/edit_field";
import EditFormField from "./EditFormField.vue";
import Unset from "@/atoms/Unset.vue";
import { ArrayWidgetDal } from "@/api/sdf/dal/edit_field";
import VueFeather from "vue-feather";
import { EditFieldService } from "@/service/edit_field";
import { ApiResponse } from "@/api/sdf";
import { UpdateFromEditFieldResponse } from "@/service/edit_field/update_from_edit_field";
import { GlobalErrorService } from "@/service/global_error";
import { defineAsyncComponent, DefineComponent } from "vue";
import type { WidgetsProps } from "./Widgets.vue";
import { ITreeOpenState } from "@/utils/edit_field_visitor";
import { AttributeContext } from "@/api/sdf/dal/attribute";
import SiLink from "@/atoms/SiLink.vue";
import { ComponentIdentification } from "@/api/sdf/dal/component";

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

const props = defineProps<{
  show: boolean;
  coreEditField: boolean;
  indentLevel: number;
  editField: EditField;
  treeOpenState: ITreeOpenState;
  componentIdentification?: ComponentIdentification;
  attributeContext?: AttributeContext;
}>();

const widget = computed<ArrayWidgetDal>(() => {
  return props.editField.widget as ArrayWidgetDal;
});

const addToArray = () => {
  if (props.attributeContext === undefined) {
    throw new Error(
      `AttributeContext is undefined when adding to array (this is a bug)`,
    );
  }

  EditFieldService.insertFromEditField({
    objectKind: props.editField.object_kind,
    objectId: props.editField.object_id,
    editFieldId: props.editField.id,
    baggage: props.editField.baggage,
    attributeContext: props.attributeContext,
  }).subscribe((response: ApiResponse<UpdateFromEditFieldResponse>) => {
    if (response.error) {
      GlobalErrorService.set(response);
    }
  });
};
</script>

<style scoped></style>
