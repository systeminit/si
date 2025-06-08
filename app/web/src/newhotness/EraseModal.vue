<template>
  <ConfirmModal
    ref="modalRef"
    title="Erase"
    irreversible
    @confirm="emit('confirm')"
  >
    <div class="flex flex-col gap-xs">
      <div>
        Erase immediately removes the component and all related data from both
        HEAD and the current change set. This is an irreversible action that may
        lead to desynchronization. Are you sure you want to proceed?
      </div>
      <div
        :class="
          clsx(
            'flex flex-row items-center gap-xs p-xs rounded',
            themeClasses('bg-neutral-200', 'bg-neutral-700'),
          )
        "
      >
        <Icon name="info-circle" />
        <div>
          To remove this component from the change set, or set it for deletion
          when changes are applied, use "Delete"
        </div>
      </div>
    </div>
  </ConfirmModal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import ConfirmModal from "./layout_components/ConfirmModal.vue";

const modalRef = ref<InstanceType<typeof ConfirmModal>>();

const open = () => modalRef.value?.open();
const close = () => modalRef.value?.close();

const emit = defineEmits<{
  (e: "confirm"): void;
}>();

defineExpose({ open, close });
</script>
