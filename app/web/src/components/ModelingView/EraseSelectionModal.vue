<template>
  <Modal ref="modalRef" :title="'Are you sure?'">
    <div class="max-h-[80vh] overflow-hidden flex flex-col gap-sm">
      <div>
        You are about to erase
        {{
          componentsStore.erasableSelectedComponents.length > 1
            ? "the following components"
            : "this component"
        }}:
      </div>
      <div class="flex-grow overflow-y-auto">
        <Stack spacing="xs">
          <ComponentCard
            v-for="component in componentsStore.erasableSelectedComponents"
            :key="component.id"
            :componentId="component.id"
          />
        </Stack>
      </div>
      <div>
        This operation ONLY deletes the components from the diagram, while not
        enqueueing any deletion actions for execution. <br />
        It could cause unwanted desynchronization between the System Initiative
        Software and the Real World. If you want to delete components alongside
        resources, use Delete instead of Erase.
      </div>

      <div class="flex gap-sm">
        <VButton icon="x" tone="shade" variant="ghost" @click="close">
          Cancel
        </VButton>
        <VButton
          class="flex-grow"
          icon="trash"
          tone="destructive"
          @click="onConfirmWipe"
        >
          Confirm
        </VButton>
      </div>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { Modal, Stack, useModal, VButton } from "@si/vue-lib/design-system";
import { onBeforeUnmount, onMounted, ref } from "vue";

import { useComponentsStore } from "@/store/components.store";
import ComponentCard from "../ComponentCard.vue";

const componentsStore = useComponentsStore();

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

function open() {
  // event is triggered regardless of selection
  // in some cases we may want to ignore it
  if (!componentsStore.erasableSelectedComponents.length) return;

  openModal();
}

async function onConfirmWipe() {
  close();
  if (componentsStore.erasableSelectedComponents.length > 0) {
    await componentsStore.DELETE_COMPONENTS(
      [...new Set(componentsStore.erasableSelectedComponents.map((c) => c.id))],
      true,
    );
  }
  componentsStore.setSelectedComponentId(null);
}

const modelingEventBus = componentsStore.eventBus;
onMounted(() => {
  modelingEventBus.on("eraseSelection", open);
  window.addEventListener("keydown", onKeyDown);
});
onBeforeUnmount(() => {
  modelingEventBus.off("eraseSelection", open);
  window.removeEventListener("keydown", onKeyDown);
});

const onKeyDown = async (e: KeyboardEvent) => {
  if (e.key === "Enter" && modalRef.value?.isOpen) {
    onConfirmWipe();
  }
};

defineExpose({ open, close });
</script>
