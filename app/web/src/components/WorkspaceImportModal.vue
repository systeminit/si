<template>
  <Modal
    ref="modalRef"
    title="Import Workspace"
    size="xl"
    :noExit="blockExit"
    noAutoFocus
  >
    <Stack>
      <template v-if="importReqStatus.isPending">
        <LoadingMessage>
          Restoring your workspace from a backup!
          <template #moreContent>
            <p class="italic font-sm">
              Please do not refresh while this in progress.
            </p>
          </template>
        </LoadingMessage>
      </template>
      <template v-else-if="importReqStatus.isSuccess">
        <p>Your workspace has been restored!</p>
        <p>To see your changes, please reload your browser</p>
        <VButton icon="refresh" @click="refreshHandler">Reload</VButton>
      </template>
      <template v-else>
        <p>
          You are about to restore from a workspace backup. Please note that all
          data currently in the local workspace will be overwritten and replaced
          with the contents of the backup.
        </p>
        <p>
          To continue, please select the backup / export you would like to
          restore from, and confirm the loss of local data:
        </p>

        <ErrorMessage :requestStatus="loadExportsReqStatus" />

        <template v-if="loadExportsReqStatus.isSuccess">
          <VormInput
            v-model="selectedBackupId"
            type="dropdown"
            label="Select an export to restore from"
            placeholder="- Select an export -"
            required
            requiredMessage="Select an export to continue"
          >
            <VormInputOption
              v-for="item in backupsList"
              :key="item.id"
              :value="item.id"
            >
              {{ item.createdAt }}
            </VormInputOption>
          </VormInput>
        </template>
        <template v-else-if="loadExportsReqStatus.isPending">
          <VormInput type="container" label="Select an export to restore from">
            <Inline alignY="center">
              <Icon name="loader" />
              <div>Loading your exports</div>
            </Inline>
          </VormInput>
        </template>

        <VormInput
          v-model="confirmedDataLoss"
          type="checkbox"
          noLabel
          required
          requiredMessage="You must check this box to continue"
        >
          I understand my local workspace data will be overwritten
        </VormInput>

        <ErrorMessage :requestStatus="importReqStatus" />

        <VButton
          icon="cloud-download"
          :disabled="!loadExportsReqStatus.isSuccess || validationState.isError"
          :requestStatus="importReqStatus"
          @click="continueHandler"
          >Restore from backup</VButton
        >
      </template>
    </Stack>
  </Modal>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import {
  ErrorMessage,
  Icon,
  Inline,
  LoadingMessage,
  Modal,
  Stack,
  VButton,
  VormInput,
  VormInputOption,
  useModal,
  useValidatedInputGroup,
} from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import { useWorkspacesStore } from "@/store/workspaces.store";

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

const { validationState, validationMethods } = useValidatedInputGroup();

const workspacesStore = useWorkspacesStore();
const loadExportsReqStatus = workspacesStore.getRequestStatus(
  "FETCH_WORKSPACE_BACKUPS",
);
const importReqStatus = workspacesStore.getRequestStatus(
  "RESTORE_WORKSPACE_BACKUP",
);

const backupsList = computed(() => workspacesStore.workspaceBackups);

const selectedBackupId = ref<string>();
const confirmedDataLoss = ref(false);

function open() {
  selectedBackupId.value = undefined;
  confirmedDataLoss.value = false;
  workspacesStore.FETCH_WORKSPACE_BACKUPS();
  openModal();
}

async function continueHandler() {
  if (validationMethods.hasError()) return;
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  await workspacesStore.RESTORE_WORKSPACE_BACKUP(selectedBackupId.value!);
}

function refreshHandler() {
  window.location.reload();
}

const blockExit = computed(
  () => importReqStatus.value.isPending || importReqStatus.value.isSuccess,
);

defineExpose({ open, close });
</script>
