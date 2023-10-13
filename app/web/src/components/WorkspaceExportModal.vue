<template>
  <Modal
    ref="modalRef"
    title="Export Workspace"
    size="lg"
    @closeComplete="closeHandler"
  >
    <Stack>
      <template v-if="exportReqStatus.isSuccess">
        <p>Export succeeded!</p>
        <p>
          You can now import this workspace by going to
          <br />
          workspace settings (gear in top right) > "Import Workspace"
        </p>
      </template>
      <template v-else>
        <p>
          You are about to export this workspace to the cloud. You will then be
          able to import it on this or another running instance of SI.
        </p>

        <p>Click the button below to continue:</p>

        <ErrorMessage :requestStatus="exportReqStatus" />
      </template>

      <VButton
        v-if="!exportReqStatus.isSuccess"
        icon="cloud-upload"
        :requestStatus="exportReqStatus"
        loadingText="Exporting your workspace..."
        @click="continueHandler"
        >Export this workspace</VButton
      >
      <VButton v-else icon="check" @click="close">Close this window</VButton>
    </Stack>
  </Modal>
</template>

<script setup lang="ts">
import {
  ErrorMessage,
  Modal,
  Stack,
  VButton,
  useModal,
} from "@si/vue-lib/design-system";
import { ref } from "vue";
import { useModuleStore } from "@/store/module.store";

const moduleStore = useModuleStore();

const exportReqStatus = moduleStore.getRequestStatus("EXPORT_WORKSPACE");

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

function open() {
  openModal();
}

async function continueHandler() {
  await moduleStore.EXPORT_WORKSPACE();
}
function closeHandler() {
  moduleStore.clearRequestStatus("EXPORT_WORKSPACE");
}

defineExpose({ open, close });
</script>
