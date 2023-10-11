<template>
  <div class="overflow-hidden">
    <div
      class="pb-md flex flex-row gap-sm align-middle items-center justify-between"
    >
      <div>
        <div class="text-lg font-bold pb-sm">Your dashboard</div>
        <div v-if="featureFlagsStore.CREATE_WORKSPACES">
          From here you can log into any of your workspaces.
        </div>
        <div v-else>
          From here you can log into your local dev instance. Eventually this
          will be where you can manage multiple workspaces, users,
          organizations, etc.
        </div>
      </div>
      <VButton
        v-if="featureFlagsStore.CREATE_WORKSPACES"
        label="Create Workspace"
        icon="plus"
        @click="showCreateWorkspaceModal"
      />
    </div>
    <template v-if="loadWorkspacesReqStatus.isPending">
      <Icon name="loader" />
    </template>
    <template v-else-if="loadWorkspacesReqStatus.isError">
      <ErrorMessage :requestStatus="loadWorkspacesReqStatus" />
    </template>
    <template v-else-if="loadWorkspacesReqStatus.isSuccess">
      <Stack>
        <WorkspaceLinkWidget
          v-for="workspace in workspaces"
          :key="workspace.id"
          :workspaceId="workspace.id"
          :editing="
            editingWorkspace?.id === workspace.id &&
            editWorkspaceModalRef.isOpen
          "
          @edit="showEditWorkspaceModal(workspace)"
        />
      </Stack>
    </template>
    <Modal
      v-if="
        featureFlagsStore.CREATE_WORKSPACES || featureFlagsStore.EDIT_WORKSPACES
      "
      ref="editWorkspaceModalRef"
      :capitalizeTitle="false"
    >
      <template #title>
        <template v-if="editWorkspaceModalCreatingNew">
          Create New Workspace
        </template>
        <template v-else>
          Edit Workspace:
          <span class="italic">{{ editingWorkspace?.displayName }}</span>
        </template>
      </template>
      <form>
        <Stack>
          <ErrorMessage
            v-if="editWorkspaceModalCreatingNew"
            :requestStatus="createWorkspaceReqStatus"
          />
          <ErrorMessage v-else :requestStatus="editWorkspaceReqStatus" />
          <VormInput
            v-model="draftWorkspace.displayName"
            label="Display Name"
            placeholder="A display name for this workspace"
            required
          />
          <VormInput
            v-model="draftWorkspace.instanceUrl"
            label="URL"
            autocomplete="url"
            placeholder="The instance url for this workspace"
            required
          />

          <VButton
            v-if="editWorkspaceModalCreatingNew"
            iconRight="chevron--right"
            :disabled="validationState.isError"
            :requestStatus="createWorkspaceReqStatus"
            loadingText="Creating..."
            tone="action"
            variant="solid"
            @click="createWorkspace"
          >
            Create Workspace
          </VButton>
          <VButton
            v-else
            iconRight="chevron--right"
            :disabled="validationState.isError"
            :requestStatus="editWorkspaceReqStatus"
            loadingText="Applying..."
            tone="action"
            variant="solid"
            @click="editWorkspace"
          >
            Apply Edits
          </VButton>
        </Stack>
      </form>
    </Modal>
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import { computed, ref, watch } from "vue";
import {
  Icon,
  VormInput,
  Stack,
  ErrorMessage,
  VButton,
  Modal,
  useValidatedInputGroup,
} from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { useAuthStore } from "@/store/auth.store";
import { Workspace, useWorkspacesStore } from "@/store/workspaces.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import WorkspaceLinkWidget from "@/components/WorkspaceLinkWidget.vue";

const authStore = useAuthStore();
const workspacesStore = useWorkspacesStore();
const featureFlagsStore = useFeatureFlagsStore();

const { validationState, validationMethods } = useValidatedInputGroup();

const createWorkspaceReqStatus =
  workspacesStore.getRequestStatus("CREATE_WORKSPACE");
const editWorkspaceReqStatus =
  workspacesStore.getRequestStatus("EDIT_WORKSPACE");

const workspaces = computed(() => workspacesStore.workspaces);
const blankWorkspace: Workspace = {
  id: "",
  instanceType: "local",
  instanceUrl: "",
  displayName: "",
  createdByUserId: authStore.user?.id ?? "",
  slug: "",
  createdAt: "",
};
const draftWorkspace = ref<Workspace>(_.cloneDeep(blankWorkspace));
const resetDraftWorkspace = () => {
  editingWorkspace.value = undefined;
  draftWorkspace.value = _.cloneDeep(blankWorkspace);
};

useHead({ title: "Dashboard" });

const loadWorkspacesReqStatus =
  workspacesStore.getRequestStatus("LOAD_WORKSPACES");

function reloadWorkspaces() {
  if (import.meta.env.SSR) return;
  if (!authStore.userIsLoggedIn) return;

  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  workspacesStore.LOAD_WORKSPACES();
}

watch(() => authStore.userIsLoggedIn, reloadWorkspaces, { immediate: true });

const editWorkspaceModalCreatingNew = ref(false);
const editWorkspaceModalRef = ref();
const editingWorkspace = ref<Workspace>();

const showCreateWorkspaceModal = () => {
  resetDraftWorkspace();
  editWorkspaceModalCreatingNew.value = true;
  editWorkspaceModalRef.value.open();
};
const createWorkspace = async () => {
  if (validationMethods.hasError()) return;

  const res = await workspacesStore.CREATE_WORKSPACE(draftWorkspace.value);

  if (res.result.success) {
    editWorkspaceModalRef.value.close();
  }
};

const showEditWorkspaceModal = (workspace: Workspace) => {
  editingWorkspace.value = workspace;
  draftWorkspace.value = _.cloneDeep(workspace);
  editWorkspaceModalCreatingNew.value = false;
  editWorkspaceModalRef.value.open();
};
const editWorkspace = async () => {
  if (validationMethods.hasError()) return;

  const res = await workspacesStore.EDIT_WORKSPACE(draftWorkspace.value);

  if (res.result.success) {
    editWorkspaceModalRef.value.close();
  }
};
</script>
