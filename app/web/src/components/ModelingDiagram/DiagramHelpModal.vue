<template>
  <Modal ref="modalRef" title="Diagram controls info">
    <ul class="space-y-xs">
      <li>Mouse - add, create, and select Components</li>
      <li>Plus / minus - adjust zoom level</li>
      <li>CMD + scrollwheel - zoom in/out</li>
      <li>Option/Alt + click - ignore clicked element, force drag to select</li>
      <li>Trackpad pinch - adjust zoom</li>
      <li>Arrow keys - Nudge selection, hold shift to nudge further</li>
      <li>Spacebar - hold to activate drag-to-scroll</li>
      <li>Esc - clear selection</li>
      <li>Backspace / delete - delete selected component</li>
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
