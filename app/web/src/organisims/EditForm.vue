<template>
  <Widgets
    v-if="editFields"
    :show="true"
    :edit-fields="editFields"
    :indent-level="1"
    :tree-open-state="{}"
  />
</template>

<script setup lang="ts">
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";
import { fromRef, refFrom } from "vuse-rx";
import { EditFieldService } from "@/service/edit_field";
import { GlobalErrorService } from "@/service/global_error";
import Widgets from "@/organisims/EditForm/Widgets.vue";
import { combineLatest, from } from "rxjs";
import { switchMap } from "rxjs/operators";
import { standardVisibilityTriggers$ } from "@/observable/visibility";

const props = defineProps<{
  objectKind: EditFieldObjectKind;
  objectId: number;
}>();

const props$ = fromRef(props, { immediate: true, deep: true });

const editFields = refFrom<EditFields>(
  combineLatest([props$, standardVisibilityTriggers$]).pipe(
    switchMap(([props, [visibility]]) => {
      return EditFieldService.getEditFields({
        id: props.objectId,
        objectKind: props.objectKind,
        ...visibility,
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
