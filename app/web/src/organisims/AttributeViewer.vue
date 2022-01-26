<template>
  <div v-if="componentId" class="flex flex-col w-full overflow-hidden">
    <div class="flex flex-col w-full overflow-hidden scrollbar">
      <EditForm
        :object-kind="EditFieldObjectKind.Component"
        :object-id="props.componentId"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import EditForm from "@/organisims/EditForm.vue";
import { EditFieldObjectKind } from "@/api/sdf/dal/edit_field";

const props = defineProps<{
  componentId: number;
}>();
</script>

<style scoped>
.scrollbar {
  -ms-overflow-style: none; /* edge, and ie */
  scrollbar-width: none; /* firefox */
}

.scrollbar::-webkit-scrollbar {
  display: none; /*chrome, opera, and safari */
}
</style>
