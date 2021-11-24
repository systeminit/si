<template>
  <Widgets :edit-fields="editFields" />
</template>

<script setup lang="ts">
import { PropType, ref } from "vue";
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";
import { untilUnmounted } from "vuse-rx";
import { EditFieldService } from "@/service/edit_field";
import { GlobalErrorService } from "@/service/global_error";
import Widgets from "@/organisims/EditForm/Widgets.vue";

const props = defineProps({
  objectKind: {
    type: String as PropType<EditFieldObjectKind>,
    required: true,
  },
  objectId: {
    type: Number,
    required: true,
  },
});

type TreeOpenState = Record<string, boolean>;
const treeOpenState = ref<TreeOpenState>({});

const editFields = ref<EditFields>([]);
untilUnmounted(
  EditFieldService.getEditFields({
    id: props.objectId,
    objectKind: props.objectKind,
  }),
).subscribe((response) => {
  if (response.error) {
    GlobalErrorService.set(response);
    editFields.value = [];
  } else {
    editFields.value = response.fields;
  }
});
</script>
