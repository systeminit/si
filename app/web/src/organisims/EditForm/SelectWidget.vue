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
      <select
        v-model="currentValue"
        class="pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey si-property disabled:opacity-50"
        :class="borderColor"
        @change="onBlur"
        @focus="onFocus"
        @blur="onBlur"
      >
        <option
          v-for="option in widget.options.options"
          :key="String(option.value)"
          :value="option.value"
        >
          {{ option.label }}
        </option>
      </select>
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
import type { EditField, SelectWidgetDal } from "@/api/sdf/dal/edit_field";
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

const widget = computed<SelectWidgetDal>(() => {
  // Lies, damn lies, and statistics!
  return props.editField.widget as SelectWidgetDal;
});

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

watch(
  () => props.editField,
  (editField, _prevEditField) => {
    if (!updating.value && editField.value != currentValue.value) {
      currentValue.value = editField.value;
    }
  },
);

const borderColor = computed((): Record<string, boolean> => {
  if (props.editField.visibility_diff.kind != "None") {
    return { "input-border-gold": true };
  } else {
    return { "input-border-grey": true };
  }
});
const textColor = computed((): Record<string, boolean> => {
  if (props.editField.visibility_diff.kind != "None") {
    return { "text-gold": true };
  } else {
    return { "text-gold": false };
  }
});
</script>

<style scoped></style>
