<template>
  <Modal ref="modalRef" title="Controls For Diagram Editor">
    <ul class="space-y-xs">
      <li>Mouse - Add, Create, and Select Components</li>
      <li>Arrow Keys - Nudge Selected Component</li>
      <li>Spacebar - Hold To Drag Scroll</li>
      <li>Plus/Minus Keys - Adjust Zoom Level</li>
      <li>Esc - Deselect Component</li>
      <li>Backspace/Delete - Delete Selected Component</li>
    </ul>
  </Modal>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import { Modal, useModal } from "@si/vue-lib/design-system";

const modalRef = ref<InstanceType<typeof Modal>>();
const { open, close } = useModal(modalRef);
defineExpose({ open, close });
</script>
