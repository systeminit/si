<template>
  <ConfirmModal
    ref="modalRef"
    title="Erase"
    irreversible
    @confirm="emit('confirm')"
  >
    <div>
      You are about to erase
      {{
        components.length > 1
          ? `these ${components.length} components`
          : "this component"
      }}:
    </div>

    <div class="scrollable">
      <ComponentCard
        v-for="component in components"
        :key="component.id"
        :component="component"
      />
    </div>

    <div>
      Erase immediately removes the component{{
        components.length > 1 ? "s" : ""
      }}
      and all related data from both HEAD and the current change set. This is an
      irreversible action that may lead to desynchronization. Are you sure you
      want to proceed?
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
  </ConfirmModal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ComponentInList } from "@/workers/types/entity_kind_types";
import ConfirmModal from "./layout_components/ConfirmModal.vue";
import ComponentCard from "./ComponentCard.vue";

const components = ref<ComponentInList[]>([]);

const modalRef = ref<InstanceType<typeof ConfirmModal>>();

function open(selectedComponents: ComponentInList[]) {
  components.value = selectedComponents;
  modalRef.value?.open();
}
function close() {
  components.value = [];
  modalRef.value?.close();
}

const emit = defineEmits<{
  (e: "confirm"): void;
}>();

defineExpose({ open, close });
</script>
