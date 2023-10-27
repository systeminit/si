<template>
  <div class="overflow-hidden">
    <template v-if="loadWorkspacesReqStatus.isSuccess">
      <div
        class="pb-md flex flex-row gap-sm align-middle items-center justify-between"
      >
        <div>
          <div class="text-lg font-bold pb-sm">
            {{ draftWorkspace.displayName || "Workspace Details" }}
          </div>
          <div v-if="featureFlagsStore.INVITE_USER">
            From here you can manage this workspace and invite users to be part
            of it.
          </div>
          <div v-else>From here you can manage this workspace</div>
        </div>
      </div>

      <Stack>
        <ErrorMessage
          :requestStatus="
            createMode ? createWorkspaceReqStatus : editWorkspaceReqStatus
          "
        />
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
          iconRight="chevron--right"
          :disabled="validationState.isError"
          :requestStatus="
            createMode ? createWorkspaceReqStatus : editWorkspaceReqStatus
          "
          :loadingText="createMode ? 'Creating...' : 'Applying...'"
          tone="action"
          variant="solid"
          @click="() => (createMode ? createWorkspace() : editWorkspace())"
        >
          {{ createMode ? "Create Workspace" : "Save" }}
        </VButton>
      </Stack>
      <div v-if="!createMode && featureFlagsStore.INVITE_USER">
        <template v-if="loadWorkspaceMembersReqStatus.isPending">
          <Icon name="loader" />
        </template>
        <template v-else-if="loadWorkspaceMembersReqStatus.isError">
          <ErrorMessage :requestStatus="loadWorkspaceMembersReqStatus" />
        </template>
        <template v-else-if="loadWorkspaceMembersReqStatus.isSuccess">
          <Stack>
            <div class="text-lg font-bold">Members of this workspace:</div>
            <div
              v-for="memUser in members"
              :key="memUser.userId"
              class="rounded p-sm grid grid-cols-2 items-center gap-sm bg-neutral-400 dark:bg-neutral-600 p-xs pr-sm"
            >
              <div class="font-bold">Email:</div>
              <div>{{ memUser.email }}</div>
              <div class="font-bold">Role:</div>
              <div>{{ memUser.role }}</div>
              <div class="font-bold">Has Accepted Invite:</div>
              <div>{{ memUser.signupAt ? "Yes" : "No" }}</div>
            </div>
          </Stack>
        </template>
      </div>
      <div v-if="featureFlagsStore.INVITE_USER && canInviteUsers" class="pt-4">
        <template v-if="inviteUserReqStatus.isPending">
          <Icon name="loader" />
        </template>
        <template v-else-if="inviteUserReqStatus.isError">
          <ErrorMessage :requestStatus="inviteUserReqStatus" />
        </template>
        <template v-if="!createMode || true">
          <Stack spacing="lg">
            <Stack>
              <VormInput
                v-model="newMember.email"
                type="email"
                label="User Email to Invite"
              />
              <VButton
                class="flex-none"
                tone="action"
                variant="solid"
                :requestStatus="inviteUserReqStatus"
                @click="inviteButtonHandler"
                >Invite User</VButton
              >
            </Stack>
          </Stack>
        </template>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import { computed, PropType, reactive, watch } from "vue";
import {
  Icon,
  VormInput,
  Stack,
  ErrorMessage,
  VButton,
  useValidatedInputGroup,
} from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { useRouter } from "vue-router";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore, WorkspaceId } from "@/store/workspaces.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const authStore = useAuthStore();
const workspacesStore = useWorkspacesStore();
const router = useRouter();
const featureFlagsStore = useFeatureFlagsStore();

const props = defineProps({
  workspaceId: { type: String as PropType<WorkspaceId>, required: true },
});

const { validationState, validationMethods } = useValidatedInputGroup();

const members = computed(() => workspacesStore.selectedWorkspaceMembers);
const blankWorkspace = {
  instanceUrl: "",
  displayName: "",
};
const draftWorkspace = reactive(_.cloneDeep(blankWorkspace));
const newMember = reactive({ email: "", role: "collaborator" });
useHead({ title: "Dashboard" });

const createWorkspaceReqStatus =
  workspacesStore.getRequestStatus("CREATE_WORKSPACE");
const editWorkspaceReqStatus =
  workspacesStore.getRequestStatus("EDIT_WORKSPACE");
const loadWorkspaceMembersReqStatus = workspacesStore.getRequestStatus(
  "LOAD_WORKSPACE_MEMBERS",
);
const inviteUserReqStatus = workspacesStore.getRequestStatus(
  "INVITE_USER",
  "email",
);

const createMode = computed(() => props.workspaceId === "new");
const canInviteUsers = computed(
  () => workspacesStore.workspacesById[props.workspaceId].role === "OWNER",
);

const loadWorkspacesReqStatus =
  workspacesStore.getRequestStatus("LOAD_WORKSPACES");

function reloadWorkspaces() {
  if (import.meta.env.SSR) return;
  if (!authStore.userIsLoggedIn) return;

  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  workspacesStore.LOAD_WORKSPACES();
}
watch(() => authStore.userIsLoggedIn, reloadWorkspaces, { immediate: true });

watch(
  [() => props.workspaceId, loadWorkspacesReqStatus],
  () => {
    if (!loadWorkspacesReqStatus.value.isSuccess) return;
    _.assign(
      draftWorkspace,
      _.cloneDeep(
        createMode.value
          ? blankWorkspace
          : workspacesStore.workspacesById[props.workspaceId],
      ),
    );
    if (!createMode.value) {
      // eslint-disable-next-line @typescript-eslint/no-floating-promises
      workspacesStore.LOAD_WORKSPACE_MEMBERS(props.workspaceId);
    }
  },
  { immediate: true },
);
const createWorkspace = async () => {
  if (validationMethods.hasError()) return;

  const res = await workspacesStore.CREATE_WORKSPACE(draftWorkspace);

  if (res.result.success) {
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    await router.push({
      name: "workspace-settings",
      params: { workspaceId: res.result.data.newWorkspaceId },
    });
  }
};
const editWorkspace = async () => {
  if (validationMethods.hasError()) return;

  const res = await workspacesStore.EDIT_WORKSPACE(draftWorkspace);

  if (res.result.success) {
    return;
  }
};

const inviteButtonHandler = async () => {
  const res = await workspacesStore.INVITE_USER(newMember, props.workspaceId);

  if (res.result.success) {
    newMember.email = "";
  }
};
</script>
