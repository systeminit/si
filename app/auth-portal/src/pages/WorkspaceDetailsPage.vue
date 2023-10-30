<template>
  <div class="overflow-hidden">
    <template v-if="loadWorkspacesReqStatus.isSuccess || createMode">
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
          :disabled="!canInviteUsers && !createMode"
        />
        <VormInput
          v-model="draftWorkspace.instanceUrl"
          label="URL"
          autocomplete="url"
          placeholder="The instance url for this workspace"
          required
          :disabled="!canInviteUsers && !createMode"
        />

        <VButton
          iconRight="chevron--right"
          :disabled="
            validationState.isError || (!canInviteUsers && !createMode)
          "
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
            <table
              class="min-w-full divide-y divide-gray-200 dark:divide-gray-700"
            >
              <thead>
                <tr>
                  <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-white-500 uppercase"
                  >
                    Email
                  </th>
                  <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-white-500 uppercase"
                  >
                    Role
                  </th>
                  <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-white-500 uppercase"
                  >
                    INVITE ACCEPTED?
                  </th>
                  <th
                    scope="col"
                    class="px-6 py-3 text-left text-xs font-medium text-white-500 uppercase"
                  />
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                <tr v-for="memUser in members" :key="memUser.userId">
                  <td
                    class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-800 dark:text-gray-200"
                  >
                    {{ memUser.email }}
                  </td>
                  <td
                    class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-800 dark:text-gray-200 normal-case"
                  >
                    {{ memUser.role }}
                  </td>
                  <td
                    class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-800 dark:text-gray-200 normal-case"
                  >
                    {{ memUser.signupAt ? "Yes" : "No" }}
                  </td>
                  <td
                    class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-800 dark:text-gray-200 normal-case"
                  >
                    <ErrorMessage :requestStatus="deleteUserHandlerReq" />
                    <div
                      v-if="memUser.role !== 'OWNER'"
                      @click="deleteUserHandler(memUser.email)"
                    >
                      <Icon name="trash" />
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
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
const newMember = reactive({ email: "", role: "editor" });
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

const deleteUserHandlerReq = workspacesStore.getRequestStatus("REMOVE_USER");
const deleteUserHandler = async (email: string) => {
  if (email === "") return;
  return await workspacesStore.REMOVE_USER(email, props.workspaceId);
};

const inviteButtonHandler = async () => {
  if (newMember.email === "") return;
  const res = await workspacesStore.INVITE_USER(newMember, props.workspaceId);

  if (res.result.success) {
    newMember.email = "";
  }
};
</script>
