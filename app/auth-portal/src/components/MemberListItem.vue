<template>
  <tr
    class="children:px-md children:py-sm children:truncate text-sm font-medium text-neutral-800 dark:text-neutral-200"
  >
    <td class="">
      <div
        class="xl:max-w-[800px] lg:max-w-[60vw] md:max-w-[50vw] sm:max-w-[40vw] max-w-[150px] truncate"
      >
        {{ memUser.email }}
      </div>
    </td>
    <td class="normal-case">
      <ErrorMessage :requestStatus="changeMembershipReq" />
      <VormInput
        v-model="changeWorkspaceRoleType"
        :options="changeWorkspaceRoleTypeDropdownOptions"
        :disabled="
          !featureFlagsStore.CHANGE_USER_ROLE ||
          memUser.role === 'OWNER' ||
          !isWorkspaceOwner
        "
        noLabel
        placeholder="Choose a role for this user"
        type="dropdown"
      />
    </td>
    <td class="normal-case">
      <ErrorMessage :requestStatus="deleteUserHandlerReq" />
      <div
        v-if="canInviteMember"
        class="cursor-pointer hover:text-destructive-500"
        @click="deleteUserHandler(memUser.email)"
      >
        <Icon name="trash" />
      </div>
    </td>
  </tr>
</template>

<script lang="ts" setup>
import { computed, PropType, ref, watch } from "vue";
import { Icon, VormInput, ErrorMessage } from "@si/vue-lib/design-system";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import {
  useWorkspacesStore,
  WorkspaceId,
  WorkspaceMember,
} from "@/store/workspaces.store";

const featureFlagsStore = useFeatureFlagsStore();
const workspacesStore = useWorkspacesStore();

const props = defineProps({
  memUser: { type: Object as PropType<WorkspaceMember>, required: true },
  draftWorkspace: {
    type: Object as PropType<{ instanceUrl: string }>,
    required: true,
  },
  workspaceId: { type: String as PropType<WorkspaceId>, required: true },
});

const isWorkspaceOwner = computed(
  () =>
    props.workspaceId === "new" ||
    workspacesStore.workspacesById[props.workspaceId]?.role === "OWNER",
);

const deleteUserHandlerReq = workspacesStore.getRequestStatus("REMOVE_USER");
const deleteUserHandler = async (email: string) => {
  if (email === "" || props.memUser.role === "OWNER") return;
  const res = await workspacesStore.REMOVE_USER(email, props.workspaceId);
  if (res.result.success) {
    if (!props.draftWorkspace.instanceUrl.includes("localhost")) {
      window.location.href = ` ${props.draftWorkspace.instanceUrl}/refresh-auth?workspaceId=${props.workspaceId}`;
    }
  }
};

const changeMembershipReq =
  workspacesStore.getRequestStatus("CHANGE_MEMBERSHIP");
async function changeMembership(newRole: string) {
  if (
    props.memUser.userId === "" ||
    props.memUser.role === "OWNER" ||
    props.memUser.role === newRole
  )
    return;
  const res = await workspacesStore.CHANGE_MEMBERSHIP(
    props.workspaceId,
    props.memUser.userId,
    newRole,
  );
  if (res.result.success) {
    if (!props.draftWorkspace.instanceUrl.includes("localhost")) {
      window.location.href = ` ${props.draftWorkspace.instanceUrl}/refresh-auth?workspaceId=${props.workspaceId}`;
    }
  }
}
type WorkspaceRoleType = "OWNER" | "APPROVER" | "EDITOR";
const changeWorkspaceRoleType = ref<WorkspaceRoleType>(
  props.memUser.role as WorkspaceRoleType,
);

const changeWorkspaceRoleTypeDropdownOptions = computed(() => {
  if (props.memUser.role === "OWNER") {
    return [
      { label: "Owner", value: "OWNER" },
      { label: "Approver", value: "APPROVER" },
      { label: "Collaborator", value: "EDITOR" },
    ];
  } else {
    return [
      { label: "Approver", value: "APPROVER" },
      { label: "Collaborator", value: "EDITOR" },
    ];
  }
});

const canInviteMember = computed(() => {
  if (props.workspaceId === "new") {
    return true;
  }

  const workspace = workspacesStore.workspacesById[props.workspaceId];
  return workspace?.role === "OWNER" || workspace?.role === "APPROVER";
});

watch(changeWorkspaceRoleType, (newRole) => changeMembership(newRole));
</script>
