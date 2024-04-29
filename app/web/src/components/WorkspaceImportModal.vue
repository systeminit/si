<template>
  <Modal
    ref="modalRef"
    title="Import Workspace"
    size="xl"
    :noExit="blockExit"
    noAutoFocus
  >
    <Stack>
      <template v-if="successfulImport">
        <p>This workspace has been replaced!</p>
        <p>Please reload your browser</p>

        <VButton icon="refresh" @click="refreshHandler">Reload</VButton>
      </template>
      <template v-else>
        <template v-if="!approvalInFlight">
          <p>
            You are about to import a workspace. Please note that all data
            currently in the local workspace will be overwritten and replaced
            with the contents of this workspace.
          </p>
          <p>
            To continue, please select the workspace you would like to import,
            and confirm the loss of local data:
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
          <ErrorMessage :message="workspaceStore.importError" />

          <VButton
            icon="cloud-download"
            :label="
              requiresVoting ? 'Begin Approval Process' : 'Import workspace'
            "
            :loadingText="
              requiresVoting ? 'Beginning Approval Process' : 'Importing...'
            "
            :requestStatus="
              requiresVoting ? beginApprovalProcessReqStatus : importReqStatus
            "
            :loading="workspaceStore.importLoading"
            @click="
              requiresVoting ? beginApprovalHandler() : importWorkspaceHandler()
            "
          />
        </template>
        <template v-if="approvalInFlight && !allApproved">
          <div
            :class="
              clsx(
                'p-sm flex items-center gap-3',
                !importedByYou && 'border-b dark:border-neutral-500',
              )
            "
          >
            <UserIcon v-if="importUser" :user="importUser" />
            <div>
              <template v-if="importedByYou">You have</template>
              <template v-else>
                <span class="italic">{{ importUser?.name }}</span> has
              </template>
              clicked the Import workspace button.
              <template v-if="importedByYou">
                There are other users online in this workspace, so they will get
                the chance to reject the import workspace workflow.
              </template>
              <template v-else>
                <div class="pt-4">
                  <span class="text-sm"
                    >The following workspace will be imported:</span
                  >
                  <ul class="text-xs indent-4">
                    <li>
                      Workspace Name:
                      {{
                        workspaceStore.workspaceImportSummary
                          ?.importedWorkspaceName
                      }}
                    </li>
                    <li>
                      Created At:
                      {{
                        workspaceStore.workspaceImportSummary
                          ?.workspaceExportCreatedAt
                      }}
                    </li>
                    <li>
                      Created By:
                      {{
                        workspaceStore.workspaceImportSummary
                          ?.workspaceExportCreatedBy
                      }}
                    </li>
                  </ul>
                </div>
              </template>
              <div class="flex w-full text-xs justify-center pt-4 gap-xs">
                <Icon name="tools" tone="warning" size="sm" /> Importing a
                workspace means replacing all the changesets in this workspace.
                They cannot be recovered. If you want to save current work
                please export your workspace now.
              </div>
            </div>
          </div>
          <div>
            <template v-if="importedByYou">
              <div class="flex w-full justify-center pb-2">
                <VButton
                  icon="tools"
                  size="sm"
                  tone="success"
                  loadingText="Importing Workspace"
                  label="Override vote and apply"
                  @click="importWorkspaceHandler"
                />
              </div>
              <div
                class="text-sm pb-2 italic text-center w-full text-neutral-400 border-b dark:border-neutral-500"
              >
                <template v-if="!allUsersVoted"
                  >Waiting on other users in the changeset to vote.</template
                >
              </div>
              <div class="pt-2">
                <div
                  v-for="(user, index) in presenceStore.users"
                  :key="index"
                  class="flex items-center pr-sm justify-center gap-4"
                >
                  <div class="min-w-0">
                    <UserCard :user="user" hideChangesetInfo />
                  </div>
                  <Icon
                    v-if="
                      workspaceStore.workspaceApprovals[user.pk] === 'Approve'
                    "
                    name="thumbs-up"
                    size="lg"
                    class="text-success-400"
                  />
                  <Icon
                    v-else-if="
                      workspaceStore.workspaceApprovals[user.pk] === 'Reject'
                    "
                    name="thumbs-down"
                    size="lg"
                    class="text-destructive-500"
                  />
                </div>
              </div>
            </template>
            <template v-else>
              <template v-if="!successfullyVoted">
                <div class="flex w-full justify-center pt-2 gap-xs">
                  <VButton
                    icon="thumbs-up"
                    variant="ghost"
                    tone="success"
                    loadingText="Approving"
                    label="Go ahead"
                    @click="importApprovalVote('Approve')"
                  />
                  <VButton
                    icon="thumbs-down"
                    variant="ghost"
                    tone="error"
                    loadingText="Rejecting"
                    label="No"
                    @click="importApprovalVote('Reject')"
                  />
                </div>
              </template>
              <template v-if="successfullyVoted">
                <div class="flex gap-4 w-full p-xs">
                  <Icon name="lock" size="lg" tone="warning" />
                  <span class="text-sm align-middle"
                    >This workspace locked until all users in it have voted on
                    the import or {{ importUser?.name }} has taken further
                    action.</span
                  >
                </div>
              </template>
            </template>
          </div>
        </template>
        <template v-if="importFlow">
          <template
            v-if="importReqStatus.isPending || workspaceStore.importLoading"
          >
            <LoadingMessage>
              Importing your workspace!
              <template #moreContent>
                <p class="italic font-sm">
                  Please do not refresh while this in progress.
                </p>
              </template>
            </LoadingMessage>
          </template>
        </template>
        <template v-if="rejectedWorkflow && importedByYou">
          <span class="text-sm pb-2"
            >One of the users in this workspace has rejected the import. You can
            either override and continue the import, above or 'Cancel' the
            import flow</span
          >
          <VButton
            label="Cancel Import Flow"
            variant="ghost"
            size="sm"
            tone="warning"
            loadingText="Cancelling Import Flow"
            @click="cancelApprovalHandler()"
          />
        </template>
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
import { computed, ref, watch } from "vue";
import clsx from "clsx";
import { useModuleStore, RemoteModuleSummary } from "@/store/module.store";
import { usePresenceStore } from "@/store/presence.store";
import UserIcon from "@/components/layout/navbar/UserIcon.vue";
import { useAuthStore } from "@/store/auth.store";
import UserCard from "@/components/layout/navbar/UserCard.vue";
import { useWorkspacesStore } from "@/store/workspaces.store";

const authStore = useAuthStore();
const presenceStore = usePresenceStore();
const workspaceStore = useWorkspacesStore();

const requiresVoting = computed(() => presenceStore.users.length > 0);

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

const { validationMethods } = useValidatedInputGroup();

const moduleStore = useModuleStore();
const loadExportsReqStatus = moduleStore.getRequestStatus(
  "LIST_WORKSPACE_EXPORTS",
);
const importReqStatus = workspaceStore.getRequestStatus(
  "BEGIN_WORKSPACE_IMPORT",
);

const exportsList = ref<RemoteModuleSummary[]>([]);
const selectedExportId = ref<string>();
const confirmedDataLoss = ref(false);
const hasSkippedEdges = ref(false);
const hasSkippedAttributes = ref(false);
const allUsersVoted = ref<boolean>();
const importFlow = ref(false);
const successfullyVoted = ref<boolean>();
const rejectedWorkflow = ref<boolean>();
const allApproved = ref<boolean>(false);
const successfulImport = ref<boolean>();

function resetState() {
  importFlow.value = false;
  rejectedWorkflow.value = false;
  allApproved.value = false;
  successfullyVoted.value = false;
  allUsersVoted.value = false;
  successfulImport.value = false;
}

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

  hasSkippedEdges.value = false;
  hasSkippedAttributes.value = false;

  resetState();
  openModal();
}

async function importApprovalVote(vote: string) {
  await workspaceStore.IMPORT_WORKSPACE_VOTE(vote);
  successfullyVoted.value = true;
}

const beginApprovalProcessReqStatus = workspaceStore.getRequestStatus(
  "BEGIN_APPROVAL_PROCESS",
);
async function beginApprovalHandler() {
  if (selectedExportId.value) {
    await workspaceStore.BEGIN_APPROVAL_PROCESS(selectedExportId.value);
    allApproved.value = false;
  }
}

async function cancelApprovalHandler() {
  await workspaceStore.CANCEL_APPROVAL_PROCESS();
  modalRef.value?.close();
  resetState();
}

async function importWorkspaceHandler() {
  if (validationMethods.hasError()) return;
  if (!selectedExportId.value) return;

  allApproved.value = true;
  importFlow.value = true;
  rejectedWorkflow.value = false;

  await workspaceStore.BEGIN_WORKSPACE_IMPORT(selectedExportId.value);
}

const importUser = computed(
  () =>
    presenceStore.usersById[
      workspaceStore.workspaceImportSummary?.importRequestedByUserPk || ""
    ],
);
const importedByYou = computed(
  () =>
    workspaceStore.workspaceImportSummary?.importRequestedByUserPk ===
    authStore.user?.pk,
);

const approvalInFlight = computed(
  () => !!workspaceStore.workspaceImportSummary,
);

watch(approvalInFlight, () => {
  if (approvalInFlight.value) {
    modalRef.value?.open();
  }
});

const importFinished = computed(() => !!workspaceStore.importCompletedAt);
watch(
  () => !!workspaceStore.importCompletedAt,
  () => {
    if (importFinished.value) {
      successfulImport.value = true;
    }
  },
);

const importCancelled = computed(() => !!workspaceStore.importCancelledAt);
watch(
  () => !!workspaceStore.importCancelledAt,
  () => {
    if (importCancelled.value) {
      modalRef.value?.close();
      resetState();
    }
  },
);

function refreshHandler() {
  window.location.reload();
}

watch(
  () => workspaceStore.workspaceApprovals,
  () => {
    if (!importedByYou.value) return;
    if (
      _.values(workspaceStore.workspaceApprovals).length !==
      presenceStore.users.length + 1
      // This is the number of other users + the person who triggered the merge
    )
      return;
    if (
      _.every(
        _.values(workspaceStore.workspaceApprovals),
        (a) => a === "Approve",
      )
    ) {
      importWorkspaceHandler();
    } else {
      rejectedWorkflow.value = true;
      allUsersVoted.value = true;
    }
  },
  {
    deep: true,
  },
);

const blockExit = computed(() => {
  if (approvalInFlight.value) {
    return !importedByYou.value;
  }
  return (
    importReqStatus.value.isPending ||
    importReqStatus.value.isSuccess ||
    workspaceStore.importLoading
  );
});

defineExpose({ open, close });
</script>
