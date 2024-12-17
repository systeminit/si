<template>
  <div
    v-if="mode !== 'error'"
    :class="
      clsx(
        'max-w-md flex flex-col gap-sm p-sm shadow-2xl',
        themeClasses('bg-neutral-100 border', 'bg-neutral-900'),
      )
    "
  >
    <div class="flex flex-col gap-2xs">
      <TruncateWithTooltip class="font-bold italic pb-2xs">
        {{ changeSetName }}
      </TruncateWithTooltip>
      <div class="font-bold">{{ modalData.title }}</div>
      <div class="text-sm italic">
        <Timestamp :date="modalData.date" showTimeIfToday size="extended" />
      </div>
    </div>
    <ErrorMessage
      :tone="modalData.messageTone"
      :icon="modalData.messageIcon"
      variant="block"
      class="rounded"
    >
      <template v-if="mode === 'requested'">
        This change set is currently locked until the approval is accepted or
        rejected.
        <template v-if="userIsApprover"
          >You can approve or reject this change set, or you
        </template>
        <template v-else>
          {{
            `${
              requesterIsYou
                ? "You can withdraw the approval request to make more changes and then request approval again, or you"
                : "You"
            } `
          }}
        </template>
        can switch to a different change set using the dropdown at the top of
        the screen.
      </template>
      <template v-else-if="mode === 'approved' || mode === 'rejected'">
        {{ requesterIsYou ? "Your" : "The" }} request to
        <span class="font-bold">Apply</span> change set
        <span class="font-bold">{{ changeSetName }}</span> was {{ mode }} by
        <span class="font-bold">{{ approverEmail + " " }}</span>

        <!-- {{ modalData.date.getTime() === new Date().getTime() ? "" : "on" }} -->
        <span class="font-bold">
          <Timestamp :date="modalData.date" showTimeIfToday size="extended" />
        </span>
        <div
          v-if="!requesterIsYou && !userIsApprover && mode === 'approved'"
          class="pt-xs"
        >
          <span class="font-bold">{{ requesterEmail }}</span> requested this
          <span class="font-bold">Apply</span> and can merge this change set.
          You can switch to a different change set using the dropdown at the top
          of the screen.
        </div>
      </template>
      <template v-else>
        ERROR - this message should not ever show. Something has gone wrong!
      </template>
    </ErrorMessage>
    <div class="flex flex-col gap-xs">
      <div class="text-sm">
        These actions will be applied to the real world:
      </div>
      <div
        class="flex-grow overflow-y-auto border border-neutral-100 dark:border-neutral-700"
      >
        <ActionsList slim kind="proposed" noInteraction />
      </div>
    </div>
    <template v-if="featureFlagStore.WORKSPACE_FINE_GRAINED_ACCESS_CONTROL">
      <div class="text-sm">
        Approvers who can satisfy the requirements:
        <ul>
          <li v-for="u in requiredApproverUsers" :key="u.id">{{ u.name }}</li>
        </ul>
      </div>
      <div v-if="approvalsSubmittedUsers.length > 0" class="text-sm">
        Users who approved:
        <ul>
          <li v-for="u in approvalsSubmittedUsers" :key="u.id">{{ u.name }}</li>
        </ul>
      </div>
    </template>
    <div
      v-if="requesterIsYou || mode === 'rejected' || userIsApprover"
      class="flex flex-row gap-sm"
    >
      <VButton
        v-if="userIsApprover && (mode === 'requested' || mode === 'approved')"
        label="Reject Request"
        tone="destructive"
        variant="ghost"
        @click="rejectHandler"
      />
      <VButton
        v-if="!userIsApprover && mode === 'approved'"
        label="Withdraw Request"
        tone="destructive"
        variant="ghost"
        @click="rejectHandler"
      />
      <VButton
        :label="modalData.buttonText"
        :tone="modalData.buttonTone"
        class="grow"
        :loading="mode === 'approved' ? applyingChangeSet : false"
        loadingText="Applying..."
        @click="confirmHandler"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import {
  VButton,
  Timestamp,
  Tones,
  ErrorMessage,
  IconNames,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { computed, ref, onBeforeMount } from "vue";
import clsx from "clsx";
import { useRoute, useRouter } from "vue-router";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useAuthStore, WorkspaceUser } from "@/store/auth.store";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import ActionsList from "./Actions/ActionsList.vue";

export type InsetApprovalModalMode =
  | "requested"
  | "approved"
  | "rejected"
  | "error";

const route = useRoute();
const router = useRouter();

const authStore = useAuthStore();
const changeSetsStore = useChangeSetsStore();
const featureFlagStore = useFeatureFlagsStore();

const applyingChangeSet = ref(false);
const changeSetName = computed(() => changeSetsStore.selectedChangeSet?.name);

/*
This is breaking on the happy path of applying a change set :)
1. the selected change set is <SHA-NOT-HEAD>
2. we apply the changeset, and set its status to applied
3. we push the router change to head, but the URL has not changed yet
4. WorkspaceModelAndView re-renders
5. Which re-mounts this component
6. selectedChangeSet is the <SHA-NOT-HEAD> b/c the url has not changed
7. and its status is applied
onMounted(() => {
  if (mode.value === "error") {
    try {
      throw Error(
        `User arrived at InsetApprovalModal with invalid changeSet status - ${changeSetsStore.selectedChangeSet?.status}`,
      );
    } catch (e) {
      reportError(e);
    }
    // dump the user back to head after reporting the error!
    window.location.href = `/w/${route.params.workspacePk}/head`;
  }
});
*/

onBeforeMount(async () => {
  if (featureFlagStore.WORKSPACE_FINE_GRAINED_ACCESS_CONTROL)
    await Promise.all([
      changeSetsStore.FETCH_APPROVAL_STATUS(
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        changeSetsStore.selectedChangeSetId!,
      ),
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      authStore.LIST_WORKSPACE_USERS(changeSetsStore.selectedWorkspacePk!),
    ]);
});
const approvalsSubmittedUserIds = computed(() => {
  const approvalData =
    changeSetsStore.changeSetsApprovalData[
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      changeSetsStore.selectedChangeSetId!
    ];
  return approvalData?.latestApprovals.map((a) => a.userPk);
});
const requiredApproverIds = computed(() => {
  const ids = new Set<string>();
  const approvalData =
    changeSetsStore.changeSetsApprovalData[
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      changeSetsStore.selectedChangeSetId!
    ];
  // everyone required
  approvalData?.requirements.forEach((r) => {
    return Object.values(r.approvingGroups).forEach((id) => {
      ids.add(id);
    });
  });
  // remove people who have voted
  approvalsSubmittedUserIds.value?.forEach((id) => {
    ids.delete(id);
  });
  return [...ids];
});
const approvalsSubmittedUsers = computed(() => {
  const users: WorkspaceUser[] = [];
  approvalsSubmittedUserIds.value?.forEach((id) => {
    const u = authStore.workspaceUsers[id];
    if (u) users.push(u);
  });
  return users;
});
const requiredApproverUsers = computed(() => {
  const users: WorkspaceUser[] = [];
  requiredApproverIds.value.forEach((id) => {
    const u = authStore.workspaceUsers[id];
    if (u) users.push(u);
  });
  return users;
});

const mode = computed(() => {
  if (
    changeSetsStore.selectedChangeSet?.status === ChangeSetStatus.NeedsApproval
  ) {
    return "requested";
  } else if (
    changeSetsStore.selectedChangeSet?.status === ChangeSetStatus.Approved
  ) {
    return "approved";
  } else if (
    changeSetsStore.selectedChangeSet?.status === ChangeSetStatus.Rejected
  ) {
    return "rejected";
  } else return "error";
});

const requesterEmail = computed(
  () => changeSetsStore.selectedChangeSet?.mergeRequestedByUser,
);
const requestDate = computed(
  () => changeSetsStore.selectedChangeSet?.mergeRequestedAt as IsoDateString,
);
const requesterIsYou = computed(
  () =>
    changeSetsStore.selectedChangeSet?.mergeRequestedByUserId ===
    authStore.user?.pk,
);
const approverEmail = computed(
  () => changeSetsStore.selectedChangeSet?.reviewedByUser,
);
const approveDate = computed(
  () => changeSetsStore.selectedChangeSet?.reviewedAt as IsoDateString,
);
const userIsApprover = computed(
  () => changeSetsStore.currentUserIsDefaultApprover,
);

const modalData = computed(() => {
  if (mode.value === "requested") {
    return {
      title: `Approval Requested by ${
        requesterIsYou.value ? "You" : requesterEmail.value
      }`,
      date: requestDate.value,
      buttonText: userIsApprover.value
        ? "Approve Request"
        : "Withdraw Approval Request",
      buttonTone: (userIsApprover.value ? "success" : "action") as Tones,
      messageTone: "warning" as Tones,
      messageIcon: "exclamation-circle" as IconNames,
    };
  } else if (mode.value === "approved") {
    return {
      title: `Approval Granted by ${approverEmail.value}`,
      date: approveDate.value,
      buttonText: "Apply Change Set",
      buttonTone: "success" as Tones,
      messageTone: "success" as Tones,
      messageIcon: "check-circle" as IconNames,
    };
  } else if (mode.value === "rejected") {
    return {
      title: `Approval Rejected by ${approverEmail.value}`,
      date: approveDate.value,
      buttonText: "Make Edits",
      buttonTone: "action" as Tones,
      messageTone: "destructive" as Tones,
      messageIcon: "exclamation-circle" as IconNames,
    };
  }

  return {
    title: "ERROR!",
    date: new Date(),
    buttonText: "Go Back To HEAD",
    buttonTone: "destructive" as Tones,
    messageTone: "destructive" as Tones,
  };
});

const confirmHandler = () => {
  if (mode.value === "requested") {
    if (userIsApprover.value) {
      changeSetsStore.APPROVE_CHANGE_SET_FOR_APPLY();
    } else if (requesterIsYou.value) {
      changeSetsStore.CANCEL_APPROVAL_REQUEST();
    }
  } else if (mode.value === "approved") {
    if (authStore.user) {
      applyingChangeSet.value = true;
      changeSetsStore.APPLY_CHANGE_SET(authStore.user.name);
    }
  } else if (mode.value === "rejected") {
    changeSetsStore.REOPEN_CHANGE_SET();
  } else {
    router.push({
      name: "change-set-home",
      params: {
        ...route.params,
        changeSetId: "head",
      },
    });
  }
};

const rejectHandler = () => {
  if (mode.value === "requested") {
    changeSetsStore.REJECT_CHANGE_SET_APPLY();
  } else if (mode.value === "approved" && userIsApprover.value) {
    changeSetsStore.REJECT_CHANGE_SET_APPLY();
  } else if (mode.value === "approved" && requesterIsYou.value) {
    changeSetsStore.CANCEL_APPROVAL_REQUEST();
  }
};
</script>
