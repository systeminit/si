<template>
  <Modal ref="modalRef" title="Export Workspace" size="lg" @closeComplete="closeHandler">
    <Stack>
      <template v-if="moduleStore.exportingWorkspaceOperationRunning">
        <p class="flex gap-1 items-center">
          <Icon name="loader" />
          Exporting Workspace
        </p>
        <p>
          This operation is being executed in the backend
          <br />
          feel free to close this modal
        </p>
        <VButton icon="check" @click="close">Close this window</VButton>
      </template>
      <template v-else-if="moduleStore.exportingWorkspaceOperationId">
        <template v-if="moduleStore.exportingWorkspaceOperationError">
          <p class="flex gap-1 items-center">
            <Icon name="x" class="text-destructive-500 dark:text-destructive-600" />

            Export Failed!
          </p>
          <p>
            {{ moduleStore.exportingWorkspaceOperationError }}
          </p>
        </template>
        <template v-else>
          <p>Export succeeded!</p>
          <p>
            You can now import this workspace by going to
            <br />
            workspace settings (gear in top right) > "Import Workspace"
          </p>
        </template>

        <VButton icon="refresh" @click="moduleStore.resetExportWorkspaceStatus"> Export Again </VButton>
      </template>

      <template v-else>
        <p>
          You are about to export this workspace to the cloud. You will then be able to import it on this or another
          running instance of SI.
        </p>

        <p>Click the button below to continue:</p>

        <ErrorMessage :requestStatus="exportReqStatus" />
        <VButton
          icon="cloud-upload"
          :requestStatus="exportReqStatus"
          loadingText="Exporting your workspace..."
          @click="continueHandler"
        >
          Export this workspace
        </VButton>
      </template>
    </Stack>
  </Modal>
</template>

<script setup lang="ts">
import { ErrorMessage, Icon, Modal, Stack, useModal, VButton } from "@si/vue-lib/design-system";
import { ref } from "vue";
import { useModuleStore } from "@/store/module.store";

const moduleStore = useModuleStore();

const exportReqStatus = moduleStore.getRequestStatus("EXPORT_WORKSPACE");

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

function open() {
  openModal();
}

function continueHandler() {
  moduleStore.EXPORT_WORKSPACE();
}

function closeHandler() {
  moduleStore.clearRequestStatus("EXPORT_WORKSPACE");
}

defineExpose({ open, close });
</script>
