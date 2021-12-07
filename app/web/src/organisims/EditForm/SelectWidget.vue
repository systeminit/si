<template>
  <EditFormField :show="show">
    <template #name>
      {{ editField.name }}
    </template>
    <template #edit>
      <select
        v-model="currentValue"
        class="pl-2 text-sm leading-tight text-gray-400 border border-solid focus:outline-none input-bg-color-grey si-property disabled:opacity-50"
        :class="borderColor"
        placeholder="text"
        @change="onBlur"
        @focus="onFocus"
        @blur="onBlur"
      >
        <option
          v-for="option in widget.options.options"
          :key="option.value"
          :value="option.value"
        >
          {{ option.label }}
        </option>
      </select>
    </template>
    <template #show>
      <span :class="textColor">{{ editField.value }}</span>
    </template>
  </EditFormField>
</template>

<script setup lang="ts">
import { computed, PropType, ref, watch } from "vue";
import type { EditField, SelectWidgetDal } from "@/api/sdf/dal/edit_field";
import EditFormField from "./EditFormField.vue";
import { EditFieldService } from "@/service/edit_field";
import { GlobalErrorService } from "@/service/global_error";
import { UpdateFromEditFieldResponse } from "@/service/edit_field/update_from_edit_field";
import { ApiResponse } from "@/api/sdf";

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

const borderColor = computed(
  (): Record<string, boolean> => {
    if (props.editField.visibility_diff.kind != "None") {
      return { "input-border-gold": true };
    } else {
      return { "input-border-grey": true };
    }
  },
);
const textColor = computed(
  (): Record<string, boolean> => {
    if (props.editField.visibility_diff.kind != "None") {
      return { "text-gold": true };
    } else {
      return { "text-gold": false };
    }
  },
);
</script>

<style scoped></style>
