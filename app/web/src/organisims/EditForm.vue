<template>
  <Widgets v-if="editFields" :edit-fields="editFields" :indent-level="1" />
</template>

<script setup lang="ts">
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";
import { fromRef, refFrom } from "vuse-rx";
import { EditFieldService } from "@/service/edit_field";
import { GlobalErrorService } from "@/service/global_error";
import Widgets from "@/organisims/EditForm/Widgets.vue";
import { combineLatest, from } from "rxjs";
import { switchMap } from "rxjs/operators";

const props = defineProps<{
  objectKind: EditFieldObjectKind;
  objectId: number;
}>();

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
