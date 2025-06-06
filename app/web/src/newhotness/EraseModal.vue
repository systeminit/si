<template>
  <ConfirmModal ref="modalRef" title="Erase" @confirm="emit('confirm')"
    ><!-- TODO(WENDY) - make the title reactive to what you are about to erase -->
    Erase immediately removes the component and all related data from both HEAD
    and the current change set. This is an irreversible action that may lead to
    desynchronization. Are you sure you want to proceed? To remove this
    component from the change set, or set it for deletion when changes are
    applied, use “Delete”.
  </ConfirmModal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import ConfirmModal from "./layout_components/ConfirmModal.vue";

const modalRef = ref<InstanceType<typeof ConfirmModal>>();

const open = () => modalRef.value?.open();
const close = () => modalRef.value?.close();

const emit = defineEmits<{
  (e: "confirm"): void;
}>();

defineExpose({ open, close });
</script>
