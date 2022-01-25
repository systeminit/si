<template>
  <Widgets v-if="editFields" :edit-fields="editFields" />
</template>

<script setup lang="ts">
import { PropType } from "vue";
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";
import { fromRef, refFrom } from "vuse-rx";
import { EditFieldService } from "@/service/edit_field";
import { GlobalErrorService } from "@/service/global_error";
import Widgets from "@/organisims/EditForm/Widgets.vue";
import { combineLatest, from } from "rxjs";
import { switchMap } from "rxjs/operators";

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

const props$ = fromRef(props, { immediate: true, deep: true });

const editFields = refFrom<EditFields>(
  combineLatest([props$]).pipe(
    switchMap(([props]) => {
      return EditFieldService.getEditFields({
        id: props.objectId,
        objectKind: props.objectKind,
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
