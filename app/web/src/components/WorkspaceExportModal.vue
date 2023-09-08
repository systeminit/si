<template>
  <Modal
    ref="modalRef"
    title="Export Workspace"
    size="lg"
    @closeComplete="closeHandler"
  >
    <Stack>
      <template v-if="exportReqStatus.isSuccess">
        <p>Success!</p>
        <p>
          You can now restore your workspace to this backup by going to
          <br />
          workspace settings (gear in top right) > "Import Workspace"
        </p>

        <VButton icon="check" @click="close">Close this window</VButton>
      </template>
      <template v-else>
        <p>
          You are about to export a backup of this workspace to the cloud. You
          will then be able to import / restore this backup on this or another
          running instance of SI.
        </p>

        <p>Click the button below to continue:</p>

        <VButton
          icon="cloud-upload"
          :requestStatus="exportReqStatus"
          loadingText="Exporting your workspace..."
          @click="continueHandler"
          >Export a backup of this workspace</VButton
        >
      </template>
    </Stack>
  </Modal>
</template>

<script setup lang="ts">
import { Modal, Stack, VButton, useModal } from "@si/vue-lib/design-system";
import { ref } from "vue";
import { useWorkspacesStore } from "@/store/workspaces.store";

const workspacesStore = useWorkspacesStore();
const exportReqStatus = workspacesStore.getRequestStatus(
  "EXPORT_WORKSPACE_BACKUP",
);

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

function open() {
  openModal();
}

function continueHandler() {
  workspacesStore.EXPORT_WORKSPACE_BACKUP();
}
function closeHandler() {
  workspacesStore.clearRequestStatus("EXPORT_WORKSPACE_BACKUP");
}

defineExpose({ open, close });
</script>
