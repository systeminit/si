<template>
  <button @click="removeFromEditField">
    <!-- NOTE(nick): we may want to consider using the backspace icon or something else -->
    <Icon class="ml-1" name="trash" />
  </button>
</template>

<script setup lang="ts">
import { AttributeContext } from "@/api/sdf/dal/attribute";
import { EditFieldService } from "@/service/edit_field";
import { ApiResponse } from "@/api/sdf";
import { RemoveFromEditFieldResponse } from "@/service/edit_field/remove_from_edit_field";
import { GlobalErrorService } from "@/service/global_error";
import Icon from "@/ui-lib/Icon.vue";
import type { EditField } from "@/api/sdf/dal/edit_field";

const props = defineProps<{
  attributeContext?: AttributeContext;
  editField: EditField;
}>();

const removeFromEditField = () => {
  if (props.attributeContext === undefined) {
    throw new Error(
      `AttributeContext is undefined when unsetting an attribute (this is a bug)`,
    );
  }

  EditFieldService.removeFromEditField({
    objectKind: props.editField.object_kind,
    objectId: props.editField.object_id,
    editFieldId: props.editField.id,
    baggage: props.editField.baggage,
    attributeContext: props.attributeContext,
  }).subscribe((response: ApiResponse<RemoveFromEditFieldResponse>) => {
    if (response.error) {
      GlobalErrorService.set(response);
    }
  });
};
</script>

<style scoped>
/* Is this what we want? It's how the demo works */
button:focus {
  outline: 1px dotted;
}
</style>
