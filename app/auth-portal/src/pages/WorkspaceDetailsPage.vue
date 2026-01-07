<template>
  <div class="overflow-hidden">
    <template v-if="loadWorkspacesReqStatus.isSuccess || createMode">
      <WorkspacePageHeader
        :subtitle="
          createMode
            ? 'Fill out this form to create a new workspace.'
            : 'From here you can manage this workspace and invite users to be part of it.'
        "
        :title="
          draftWorkspace.displayName ||
          (createMode ? 'Create New Workspace' : 'Workspace Details')
        "
      >
        <template v-if="isDefaultWorkspace">
          <div
            :class="
              clsx(
                'rounded text-sm px-xs py-2xs my-2xs w-fit',
                themeClasses(
                  'bg-success-600 text-shade-0',
                  'bg-success-500 text-shade-100',
                ),
              )
            "
          >
            DEFAULT
          </div>
        </template>
        <div
          v-if="featureFlagStore.APPROVALS_OPT_IN_OUT"
          :class="
            clsx(
              'rounded text-sm px-xs py-2xs my-2xs w-fit',
              themeClasses(
                'bg-success-600 text-shade-0',
                'bg-success-500 text-shade-100',
              ),
            )
          "
        >
          Approvals {{ approvalStatus }}
        </div>

        <IconButton
          v-if="!createMode"
          class="flex-none"
          icon="key-tilted"
          iconIdleTone="shade"
          size="lg"
          tooltip="API Tokens"
          tooltipPlacement="top"
          @click="openApiTokens"
        />
      </WorkspacePageHeader>

      <Stack>
        <ErrorMessage
          :requestStatus="
            createMode ? createWorkspaceReqStatus : editWorkspaceReqStatus
          "
        />
        <VormInput
          v-model="draftWorkspace.displayName"
          :disabled="!isWorkspaceOwner && !createMode"
          :maxLength="MAX_LENGTH_STANDARD"
          label="Display Name"
          placeholder="A display name for this workspace"
          required
          :regex="DOMAIN_FRIENDLY_INPUT_REGEX"
        />
        <VormInput
          v-if="createMode"
          v-model="createWorkspaceType"
          :options="createWorkspaceTypeDropdownOptions"
          label="Workspace Type"
          placeholder="Choose what kind of workspace to create"
          required
          type="dropdown"
        />

        <VormInput
          v-if="createWorkspaceType === 'url'"
          ref="urlInputRef"
          v-model="draftWorkspace.instanceUrl"
          :disabled="!isWorkspaceOwner"
          autocomplete="url"
          label="URL"
          placeholder="The instance url for this workspace"
          :regex="ALLOWED_URL_REGEX"
          required
        />

        <VormInput
          v-model="draftWorkspace.description"
          :disabled="!isWorkspaceOwner && !createMode"
          :required="false"
          :maxLength="MAX_LENGTH_EXTENDED"
          label="Description"
          placeholder="A description for this workspace"
          :regex="DOMAIN_FRIENDLY_INPUT_REGEX"
        />

        <div class="flex flex-row flex-wrap items-center w-full gap-xs">
          <VButton
            v-if="!createMode || createWorkspaceType"
            :class="
              clsx(
                'basis-[calc(75%-0.5rem)]',
                createMode ? 'flex-grow' : 'flex-grow-0',
              )
            "
            :disabled="
              validationState.isError || (!isWorkspaceOwner && !createMode)
            "
            :loadingText="createMode ? 'Creating...' : 'Applying...'"
            :requestStatus="
              createMode ? createWorkspaceReqStatus : editWorkspaceReqStatus
            "
            iconRight="chevron--right"
            tone="action"
            variant="solid"
            @click="() => (createMode ? createWorkspace() : editWorkspace())"
          >
            {{ createMode ? "Create Workspace" : "Save Workspace" }}
          </VButton>
          <VButton
            v-if="!createMode && featureFlagStore.APPROVALS_OPT_IN_OUT"
            class="basis-[calc(25%-0.5rem)] flex-grow-0"
            iconRight="chevron--right"
            loadingText="Changing approval status..."
            tone="action"
            variant="solid"
            :label="
              draftWorkspace.approvalsEnabled
                ? 'Disable Workspace Approvals'
                : 'Enable Workspace Approvals'
            "
            @click="changeApprovalsStatus"
          />
        </div>
      </Stack>
      <div v-if="!createMode" class="mt-sm">
        <template v-if="loadWorkspaceMembersReqStatus.isPending">
          <Icon name="loader" />
        </template>
        <template v-else-if="loadWorkspaceMembersReqStatus.isError">
          <ErrorMessage :requestStatus="loadWorkspaceMembersReqStatus" />
        </template>
        <template v-else-if="loadWorkspaceMembersReqStatus.isSuccess">
          <div class="relative">
            <Stack>
              <div class="text-lg font-bold">Members of this workspace:</div>

              <table
                class="w-full divide-y divide-neutral-400 dark:divide-neutral-600 border-b border-neutral-400 dark:border-neutral-600"
              >
                <thead>
                  <tr
                    class="children:pb-xs children:px-md children:font-bold text-left text-xs uppercase"
                  >
                    <th scope="col">Email</th>
                    <th scope="col">Role</th>
                    <th scope="col">Remove User</th>
                  </tr>
                </thead>
                <tbody
                  class="divide-y divide-neutral-300 dark:divide-neutral-700"
                >
                  <MemberListItem
                    v-for="memUser in members"
                    :key="memUser.userId"
                    :draftWorkspace="draftWorkspace"
                    :memUser="memUser"
                    :workspaceId="workspaceId"
                  />
                </tbody>
              </table>
            </Stack>
          </div>
        </template>
      </div>
      <div v-if="!createMode" class="pt-md">
        <Stack>
          <ErrorMessage
            :requestStatus="inviteUserReqStatus"
            :message="inviteUserReqStatus.error?.data.error"
          />

          <VormInput
            ref="newMemberEmailInput"
            v-model="newMember.email"
            label="User Email to Grant Workspace Access"
            type="email"
            @enterPressed="inviteButtonHandler"
          />
          <VButton
            :disabled="!canInviteMember"
            :requestStatus="inviteUserReqStatus"
            class="flex-none"
            tone="action"
            variant="solid"
            @click="inviteButtonHandler"
          >
            Add User To Workspace
          </VButton>
          <div
            v-if="latestInviteEmail"
            class="p-sm border border-neutral-400 rounded-lg transition-opacity"
          >
            we have notified {{ latestInviteEmail }} that you invited them to
            collaborate on this workspace. They will be able to see this
            workspace in their workspace list.
          </div>
        </Stack>
      </div>
      <div v-if="!createMode" class="flex justify-between items-center pt-md">
        <div class="flex flex-row gap-xs">
          <VButton
            v-if="isWorkspaceOwner"
            :disabled="!isWorkspaceOwner"
            :requestStatus="deleteWorkspaceReqStatus"
            iconRight="chevron--right"
            loadingText="Deleting..."
            tone="action"
            variant="solid"
            @click="() => deleteWorkspace()"
          >
            Delete Workspace
          </VButton>
          <VButton
            v-else
            :requestStatus="leaveWorkspaceReqStatus"
            iconRight="chevron--right"
            loadingText="Leaving..."
            tone="destructive"
            variant="solid"
            @click="() => leaveWorkspace()"
          >
            Leave Workspace
          </VButton>
        </div>
        <VButton
          label="Go to workspace"
          tone="action"
          variant="solid"
          @click="() => launchWorkspace()"
        >
        </VButton>
      </div>
    </template>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType, reactive, ref, watch, onMounted } from "vue";
import {
  Icon,
  VormInput,
  Stack,
  ErrorMessage,
  VButton,
  useValidatedInputGroup,
  IconButton,
  themeClasses,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useHead } from "@vueuse/head";
import { useRouter } from "vue-router";
import { useWorkspacesStore, WorkspaceId } from "@/store/workspaces.store";
import { useAuthStore } from "@/store/auth.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { tracker } from "@/lib/posthog";
import { API_HTTP_URL } from "@/store/api";
import MemberListItem from "@/components/MemberListItem.vue";
import WorkspacePageHeader from "@/components/WorkspacePageHeader.vue";
import {
  DOMAIN_FRIENDLY_INPUT_REGEX,
  ALLOWED_URL_REGEX,
  MAX_LENGTH_STANDARD,
  MAX_LENGTH_EXTENDED,
} from "@/lib/validations";

const workspacesStore = useWorkspacesStore();
const router = useRouter();
const authStore = useAuthStore();
const featureFlagStore = useFeatureFlagsStore();

const props = defineProps({
  workspaceId: { type: String as PropType<WorkspaceId>, required: true },
});

const urlInputRef = ref<InstanceType<typeof VormInput>>();

const { validationState, validationMethods } = useValidatedInputGroup();
const members = computed(() => {
  const members = workspacesStore.selectedWorkspaceMembers;

  return members.slice().sort((a, b) => {
    // "OWNER" should come first
    if (a.role === "OWNER" && b.role !== "OWNER") {
      return -1;
    }
    if (a.role !== "OWNER" && b.role === "OWNER") {
      return 1;
    }
    return a.email.localeCompare(b.email);
  });
});

const blankWorkspace = {
  instanceUrl: "",
  displayName: "",
  isDefault: false,
  description: "",
  isFavourite: false,
  isHidden: false,
  approvalsEnabled: false,
};
const draftWorkspace = reactive(_.cloneDeep(blankWorkspace));
const newMember = reactive({ email: "", role: "editor" });
useHead({ title: "Workspace Details" });

const createWorkspaceReqStatus =
  workspacesStore.getRequestStatus("CREATE_WORKSPACE");
const editWorkspaceReqStatus =
  workspacesStore.getRequestStatus("EDIT_WORKSPACE");
const loadWorkspaceMembersReqStatus = workspacesStore.getRequestStatus(
  "LOAD_WORKSPACE_MEMBERS",
);
const inviteUserReqStatus = workspacesStore.getRequestStatus("INVITE_USER");
const deleteWorkspaceReqStatus =
  workspacesStore.getRequestStatus("DELETE_WORKSPACE");
const leaveWorkspaceReqStatus =
  workspacesStore.getRequestStatus("LEAVE_WORKSPACE");

const createMode = computed(() => props.workspaceId === "new");
const isWorkspaceOwner = computed(
  () =>
    props.workspaceId === "new" ||
    workspacesStore.workspacesById[props.workspaceId]?.role === "OWNER",
);

const canInviteMember = computed(() => {
  if (props.workspaceId === "new") {
    return true;
  }

  const workspace = workspacesStore.workspacesById[props.workspaceId];
  return workspace?.role === "OWNER" || workspace?.role === "APPROVER";
});

const isDefaultWorkspace = computed(
  () =>
    props.workspaceId === "new" ||
    workspacesStore.workspacesById[props.workspaceId]?.isDefault,
);

const setDefaultReqStatus = workspacesStore.getRequestStatus(
  "SET_DEFAULT_WORKSPACE",
);

// Reload the workspace on page load, when user logs in, when workspace ID changes or
// default workspace is set (TODO last one probably not needed, in which case a computed
// value for workspace will suffice rather than a watch)
const loadWorkspacesReqStatus = workspacesStore.refreshWorkspaces();
watch(
  [() => props.workspaceId, setDefaultReqStatus],
  async () => {
    if (!setDefaultReqStatus.value.isSuccess) return;
    await workspacesStore.LOAD_WORKSPACES();
  },
  { immediate: true },
);

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
// const setDefaultWorkspace = async () => {
//   if (!props.workspaceId) return;

//   await workspacesStore.SET_DEFAULT_WORKSPACE(props.workspaceId);
// };
const createWorkspace = async () => {
  if (!draftWorkspace.description) {
    draftWorkspace.description = "";
  }
  if (createWorkspaceType.value === "saas") {
    draftWorkspace.instanceUrl = "https://app.systeminit.com";
  } else if (createWorkspaceType.value === "local") {
    draftWorkspace.instanceUrl = "http://localhost:8080";
  } else {
    if (draftWorkspace.instanceUrl.includes("app.systeminit")) {
      // Can't create a Remote URL workspace with our URL!
      urlInputRef.value?.setError(
        'You cannot use an "app.systeminit" URL for a Remote URL Workspace. Use "Managed By System Initiative" instead.',
      );
      return;
    } else if (
      draftWorkspace.instanceUrl.includes("localhost") ||
      draftWorkspace.instanceUrl.includes("127.0.0.1")
    ) {
      // Can't create a Remote URL workspace with localhost!
      urlInputRef.value?.setError(
        'You cannot use a "localhost" URL for a Remote URL Workspace. Use "Local Dev Instance" instead.',
      );
      return;
    }
  }

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

const openApiTokens = async () => {
  await router.push({
    name: "workspace-api-tokens",
    params: { workspaceId: props.workspaceId },
  });
};

const changeApprovalsStatus = async () => {
  if (!props.workspaceId) return;

  let newStatus = false;
  if (!draftWorkspace.approvalsEnabled) {
    newStatus = true;
  }

  const res = await workspacesStore.CHANGE_WORKSPACE_APPROVAL_STATUS(
    props.workspaceId,
    newStatus,
  );

  draftWorkspace.approvalsEnabled = newStatus;

  if (res.result.success) {
    if (!draftWorkspace.instanceUrl.includes("localhost")) {
      window.location.href = `${draftWorkspace.instanceUrl}/refresh-auth?workspaceId=${props.workspaceId}`;
    }
  }
};
const approvalStatus = computed(() => {
  if (!props.workspaceId) return "disabled";

  if (draftWorkspace.approvalsEnabled) return "enabled";

  return "disabled";
});

const deleteWorkspace = async () => {
  const res = await workspacesStore.DELETE_WORKSPACE(props.workspaceId);
  if (res.result.success) {
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    await router.push({
      name: "workspaces",
      params: {},
    });
  }
};

const leaveWorkspace = async () => {
  const res = await workspacesStore.LEAVE_WORKSPACE(props.workspaceId);
  if (res.result.success) {
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    await router.push({
      name: "workspaces",
      params: {},
    });
  }
};

const launchWorkspace = async () => {
  if (props.workspaceId) {
    tracker.trackEvent("workspace_launcher_widget_click");
    window.location.href = `${API_HTTP_URL}/workspaces/${props.workspaceId}/go`;
  }
};

const latestInviteEmail = ref<string | undefined>();
const newMemberEmailInput = ref<InstanceType<typeof VormInput>>();
const inviteButtonHandler = async () => {
  if (!newMember.email || newMember.email === "") return;
  if (newMemberEmailInput.value?.validationState.isInvalid === true) return;
  const res = await workspacesStore.INVITE_USER(newMember, props.workspaceId);

  if (res.result.success) {
    latestInviteEmail.value = newMember.email;
    newMember.email = "";

    setTimeout(() => {
      latestInviteEmail.value = undefined;
    }, 20000);
  }
};

type WorkspaceType = "saas" | "local" | "url";

const createWorkspaceType = ref<WorkspaceType | undefined>("saas");
const createWorkspaceTypeDropdownOptions = [
  { label: "Managed By System Initiative", value: "saas" },
  { label: "Local Dev Instance", value: "local" },
  { label: "Remote URL", value: "url" },
];

const user = computed(() => authStore.user);

onMounted(() => {
  if (!user.value || !user.value?.emailVerified) {
    return router.push({ name: "profile" });
  }
});
</script>
