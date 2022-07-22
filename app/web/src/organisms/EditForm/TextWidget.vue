<template>
  <!-- NOTE(nick): "EditFormField" is the same as "Field" from demo v0.4 -->
  <EditFormField
    :show="show"
    :core-edit-field="coreEditField"
    :validation-errors="props.editField.validation_errors"
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
      <input
        v-model="currentValue"
        class="appearance-none block bg-gray-900 text-gray-100 w-full px-3 py-2 border rounded-sm shadow-sm placeholder-gray-700 focus:outline-none focus:ring-indigo-200 focus:border-indigo-200 sm:text-sm"
        :class="inputStyles"
        type="text"
        aria-label="name"
        placeholder="text"
        @keyup.enter="onKeyEnter($event)"
        @focus="onFocus"
        @blur="onBlur"
      />
      <div
        v-if="!coreEditField"
        class="flex flex-row items-center w-10 ml-1 bg-red"
      >
        <Unset
          :edit-field="props.editField"
          :attribute-context="props.attributeContext"
        />
      </div>
    </template>
    <template #show>
      <span :class="textColor">{{ props.editField.value }}</span>
    </template>
  </EditFormField>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { EditField } from "@/api/sdf/dal/edit_field";
import EditFormField from "./EditFormField.vue";
import Unset from "@/atoms/Unset.vue";
import { EditFieldService } from "@/service/edit_field";
import { GlobalErrorService } from "@/service/global_error";
import { UpdateFromEditFieldResponse } from "@/service/edit_field/update_from_edit_field";
import { ApiResponse } from "@/api/sdf";
import { AttributeContext } from "@/api/sdf/dal/attribute";
import SiLink from "@/atoms/SiLink.vue";
import VueFeather from "vue-feather";

const props = defineProps<{
  show: boolean;
  coreEditField: boolean;
  editField: EditField;
  attributeContext: AttributeContext;
}>();

const updating = ref(false);
const startValue = ref(props.editField.value);
const currentValue = ref(props.editField.value);
const onFocus = () => {
  updating.value = true;
  startValue.value = currentValue.value;
};
const onBlur = () => {
  if (currentValue.value != startValue.value) {
    EditFieldService.updateFromEditField({
      objectKind: props.editField.object_kind,
      objectId: props.editField.object_id,
      editFieldId: props.editField.id,
      value: currentValue.value,
      baggage: props.editField.baggage,
      attributeContext: props.attributeContext,
    }).subscribe((response: ApiResponse<UpdateFromEditFieldResponse>) => {
      if (response.error) {
        GlobalErrorService.set(response);
      }
    });
  }
  updating.value = false;
};
const onKeyEnter = (event: KeyboardEvent) => {
  if (event?.target instanceof HTMLElement) {
    event.target.blur();
  }
};

watch(
  () => props.editField,
  (editField, _prevEditField) => {
    if (!updating.value && editField.value != currentValue.value) {
      currentValue.value = editField.value;
    }
  },
);

const inputStyles = computed((): Record<string, boolean> => {
  let styles: Record<string, boolean> = {};

  if (props.editField.visibility_diff.kind != "None") {
    styles["border-yellow-600"] = true;
    styles["border-grey-600"] = false;
  } else {
    styles["border-yellow-600"] = false;
    styles["border-grey-600"] = true;
  }
  if (props.coreEditField) {
    styles["flex-grow"] = true;
  } else {
    styles["flex-grow"] = false;
  }

  return styles;
});
const textColor = computed((): Record<string, boolean> => {
  if (props.editField.visibility_diff.kind != "None") {
    return { "text-gold": true };
  } else {
    return { "text-gold": false };
  }
});
</script>
