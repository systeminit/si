<template>
  <Modal
    ref="modalRef"
    title="Invite User"
    size="md"
    @closeComplete="closeHandler"
  >
    <Stack>
      <template v-if="inviteReqStatus.isSuccess">
        <p>Invite sent!</p>
        <VButton icon="check" @click="close">Close this window</VButton>
      </template>
      <template v-else>
        <SiTextBox id="invite-email" v-model="email" title="Invite User" />

        <ErrorMessage :requestStatus="inviteReqStatus" />
        <VButton
          icon="user-circle"
          :requestStatus="inviteReqStatus"
          loadingText="Inviting user..."
          @click="continueHandler"
          >Invite User</VButton
        >
      </template>
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
import { useWorkspacesStore } from "@/store/workspaces.store";
import SiTextBox from "@/components/SiTextBox.vue";

const email = ref("");
const workspaceStore = useWorkspacesStore();

const inviteReqStatus = workspaceStore.getRequestStatus("INVITE_USER");

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

function open() {
  openModal();
}

function continueHandler() {
  workspaceStore.INVITE_USER(email.value.trim());
  email.value = "";
}
function closeHandler() {
  workspaceStore.clearRequestStatus("INVITE_USER");
}

defineExpose({ open, close });
</script>
