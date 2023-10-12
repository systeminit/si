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
          Importing your workspace!
          <template #moreContent>
            <p class="italic font-sm">
              Please do not refresh while this in progress.
            </p>
          </template>
        </LoadingMessage>
      </template>
      <template v-else-if="importReqStatus.isSuccess">
        <p>Your workspace has been imported!</p>
        <p>To see your changes, please reload your browser</p>
        <VButton icon="refresh" @click="refreshHandler">Reload</VButton>
      </template>
      <template v-else>
        <p>
          You are about to import a workspace. Please note that all data
          currently in the local workspace will be overwritten and replaced with
          the contents of this workspace.
        </p>
        <p>
          To continue, please select the workspace you would like to import, and
          confirm the loss of local data:
        </p>

        <ErrorMessage :requestStatus="loadExportsReqStatus" />

        <template v-if="loadExportsReqStatus.isSuccess">
          <VormInput
            v-model="selectedExportId"
            type="dropdown"
            label="Select workspace"
            placeholder="- Select a workspace -"
            required
            requiredMessage="Select a workspace to continue"
          >
            <VormInputOption
              v-for="item in exportsList"
              :key="item.id"
              :value="item.id"
            >
              {{ item.name }} ({{ item.createdAt }})
            </VormInputOption>
          </VormInput>
        </template>
        <template v-else-if="loadExportsReqStatus.isPending">
          <VormInput type="container" label="Select workspace">
            <Inline alignY="center">
              <Icon name="loader" />
              <div>Loading your workspace exports</div>
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
          >Import workspace</VButton
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
import { useModuleStore, RemoteModuleSummary } from "@/store/module.store";

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

const { validationState, validationMethods } = useValidatedInputGroup();

const moduleStore = useModuleStore();
const loadExportsReqStatus = moduleStore.getRequestStatus(
  "LIST_WORKSPACE_EXPORTS",
);
const importReqStatus = moduleStore.getRequestStatus("INSTALL_REMOTE_MODULE");

const exportsList = ref<RemoteModuleSummary[]>([]);

const selectedExportId = ref<string>();
const confirmedDataLoss = ref(false);

async function open() {
  selectedExportId.value = undefined;
  confirmedDataLoss.value = false;
  const exportResponse = await moduleStore.LIST_WORKSPACE_EXPORTS();
  if (exportResponse.result.success) {
    exportsList.value = exportResponse.result.data.modules.map((workspace) => ({
      ...workspace,
      hash: workspace.latestHash,
      hashCreatedAt: workspace.latestHashCreatedAt,
    }));
  }
  openModal();
}

async function continueHandler() {
  if (validationMethods.hasError()) return;
  if (selectedExportId.value) {
    await moduleStore.INSTALL_REMOTE_MODULE(selectedExportId.value);
  }
}

function refreshHandler() {
  window.location.reload();
}

const blockExit = computed(
  () => importReqStatus.value.isPending || importReqStatus.value.isSuccess,
);

defineExpose({ open, close });
</script>
