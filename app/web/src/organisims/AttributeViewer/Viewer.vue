<template>
  <div v-if="componentId">
    <EditForm
      :object-kind="EditFieldObjectKind.Component"
      :object-id="props.componentId"
    />
  </div>
</template>

<script setup lang="ts">
import EditForm from "@/organisims/EditForm.vue";
import { EditFieldObjectKind } from "@/api/sdf/dal/edit_field";

const props = defineProps<{
  componentId: number;
}>();
</script>
