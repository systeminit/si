<template>
  <div>
    <!-- this modal is for the voting process -->
    <Modal
      ref="votingModalRef"
      :title="
        votingKind === 'merge' ? 'Apply Change Set?' : 'Abandon Change Set?'
      "
      :noExit="!canCloseModal"
    >
      <div class="max-h-[80vh] overflow-hidden flex flex-col">
        <template v-if="currentlyVoting">
          <div :class="clsx('px-sm pb-xs pt-0 flex items-center gap-sm')">
            <UserIcon
              v-if="userWhoStartedVoting"
              :user="userWhoStartedVoting"
            />
            <Icon v-else name="loader" />
            <div>
              <template v-if="votingStartedByYou">You have</template>
              <template v-else>
                <span class="italic">{{ userWhoStartedVoting?.name }}</span> has
              </template>
              clicked the
              {{ votingKind === "merge" ? "Apply" : "Abandon" }} Change Set
              button to
              {{
                votingKind === "merge"
                  ? " apply this change set to Head."
                  : "abandon this change set."
              }}
              <template v-if="votingStartedByYou">
                There are other users online in this change set, so they will
                get the chance to vote on this
                {{ votingKind === "merge" ? "apply" : "abandonment" }}.
              </template>
            </div>
          </div>
          <template v-if="votingStartedByYou">
            <div
              class="text-sm py-xs px-sm w-full bg-neutral-700 dark:text-neutral-200 text-white font-bold rounded-t-md"
            >
              <template v-if="!allUsersVoted">
                <template v-if="usersInChangeSet.length > 0">
                  {{ usersInChangeSet.length }} user{{
                    usersInChangeSet.length === 1 ? "" : "s"
                  }}
                  can vote
                </template>
                <template v-else>All voting users have left</template>
              </template>
            </div>
            <div
              class="overflow-y-auto flex-grow border dark:border-neutral-700 dark:bg-neutral-600 bg-neutral-200"
            >
              <div
                v-for="(user, index) in usersInChangeSet"
                :key="index"
                class="flex items-center pr-sm justify-between gap-4"
              >
                <div class="min-w-0">
                  <UserCard :user="user" hideChangesetInfo hideStatus />
                </div>
                <!-- while waiting show all options dimmer, once voted only show the selected vote -->
                <div class="grow flex justify-end items-center">
                  <div
                    v-if="!changeSetsStore.changeSetApprovals[user.pk]"
                    class="italic leading-tight pr-sm"
                  >
                    waiting...
                  </div>
                  <Icon
                    name="thumbs-up"
                    size="lg"
                    class="text-success-400"
                    :class="{
                      'opacity-25':
                        !changeSetsStore.changeSetApprovals[user.pk],
                      'opacity-0':
                        changeSetsStore.changeSetApprovals[user.pk] &&
                        changeSetsStore.changeSetApprovals[user.pk] !==
                          'Approve',
                    }"
                  />
                  <Icon
                    name="minus"
                    size="lg"
                    class="text-warning-400"
                    :class="{
                      'opacity-25':
                        !changeSetsStore.changeSetApprovals[user.pk],
                      'opacity-0':
                        changeSetsStore.changeSetApprovals[user.pk] &&
                        changeSetsStore.changeSetApprovals[user.pk] !== 'Pass',
                    }"
                  />
                  <Icon
                    name="thumbs-down"
                    size="lg"
                    class="text-destructive-500"
                    :class="{
                      'opacity-25':
                        !changeSetsStore.changeSetApprovals[user.pk],
                      'opacity-0':
                        changeSetsStore.changeSetApprovals[user.pk] &&
                        changeSetsStore.changeSetApprovals[user.pk] !==
                          'Reject',
                    }"
                  />
                </div>
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
                  label="Approve"
                  :disabled="statusStoreUpdating"
                  @click="vote('Approve')"
                />
                <VButton
                  icon="minus"
                  variant="ghost"
                  tone="warning"
                  loadingText="Passing"
                  label="Pass"
                  :disabled="statusStoreUpdating"
                  @click="vote('Pass')"
                />
                <VButton
                  icon="thumbs-down"
                  variant="ghost"
                  tone="destructive"
                  loadingText="Rejecting"
                  label="Reject"
                  :disabled="statusStoreUpdating"
                  @click="vote('Reject')"
                />
              </div>
            </template>
            <template v-if="successfullyVoted">
              <div class="flex gap-sm w-full p-xs">
                <Icon name="lock" size="lg" tone="warning" />
                <span class="text-sm align-middle">
                  This change set is locked until all users in the it have voted
                  on the {{ votingKind }}.
                </span>
              </div>
              <div class="grid grid-flow-col justify-center w-full">
                <RouterLink
                  :to="{
                    name: 'workspace-single',
                    params: {
                      ...route.params,
                      changeSetId: 'head',
                    },
                  }"
                  class="border border-transparent dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300 p-xs"
                >
                  Go to head
                </RouterLink>
              </div>
            </template>
            <!-- TODO(Wendy) - these are temporary buttons to help mitigate any voting bugs! -->
            <VButton
              :icon="votingKind === 'merge' ? 'tools' : 'trash'"
              tone="warning"
              :label="`Ignore Voting And ${
                votingKind === 'merge' ? 'Apply' : 'Abandon'
              } Change Set Now`"
              class="mt-sm"
              @click="completeVoting"
            />
          </template>
        </template>
        <template v-else-if="changeSet?.status === ChangeSetStatus.Open">
          <template v-if="!hasActions">
            <div class="text-md mb-xs">
              <template v-if="votingKind === 'merge'">
                Applying this change set may create, modify, or destroy real
                resources in the cloud.
              </template>
              <template v-else>
                Are you sure that you want to abandon change set
                <span class="italic font-bold">
                  {{ selectedChangeSetName }}
                </span>
                and return to HEAD?
              </template>
            </div>
            <div class="text-sm mb-sm">
              <template v-if="votingKind === 'merge'">
                Are you sure you want to apply change set
                <span class="italic font-bold">
                  {{ selectedChangeSetName }}?
                </span>
              </template>
              <template v-else>
                Once abandoned, a change set cannot be recovered.
              </template>
            </div>
          </template>
          <template v-if="hasActions">
            <div class="text-md mb-xs">
              Applying this change set may create, modify, or destroy real
              resources in the cloud.
            </div>
            <!-- TODO(Wendy) - this panel no longer allows you to select which actions to apply -->
            <div class="text-sm mb-sm">
              These actions will be applied to the real world:
            </div>
            <div
              class="flex-grow overflow-y-auto mb-sm border border-neutral-100 dark:border-neutral-700"
            >
              <ActionsList slim kind="proposed" noInteraction />
            </div>
          </template>
          <div class="flex flex-row items-center w-full gap-sm">
            <VButton
              label="Cancel"
              variant="ghost"
              tone="warning"
              icon="x"
              @click="cancelVotingProcess"
            />
            <template v-if="!changeSetsStore.headSelected">
              <VButton
                v-if="votingKind === 'merge'"
                ref="applyButtonRef"
                class="flex-grow"
                icon="tools"
                tone="success"
                :loadingText="
                  requiresVoting
                    ? 'Beginning Approval Flow'
                    : 'Applying Change Set'
                "
                :label="
                  requiresVoting ? 'Begin Approval Flow' : 'Apply Changes'
                "
                :requestStatus="
                  requiresVoting
                    ? beginApprovalReqStatus
                    : doVotingActionReqStatus
                "
                :disabled="statusStoreUpdating"
                @click="requiresVoting ? beginVoting() : completeVoting"
              />
              <VButton
                v-else
                :label="
                  requiresVoting ? 'Begin Approval Flow' : 'Abandon Change Set'
                "
                tone="destructive"
                class="flex-grow"
                icon="trash"
                :requestStatus="
                  requiresVoting
                    ? beginApprovalReqStatus
                    : doVotingActionReqStatus
                "
                :loadingText="
                  requiresVoting
                    ? 'Beginning Approval Flow'
                    : 'Abandoning Change Set'
                "
                @click="requiresVoting ? beginVoting() : completeVoting"
              />
            </template>
          </div>
        </template>
        <template v-if="rejectedWorkflow && votingStartedByYou">
          <div class="text-sm mt-xs">
            One of the users in this changeset has rejected the
            {{ votingKind }}. You can either override the vote to force the
            {{ votingKind }} to happen anyway or you can cancel.
          </div>
        </template>
        <div
          v-if="currentlyVoting && votingStartedByYou"
          class="flex w-full justify-center gap-sm mt-xs"
        >
          <VButton
            label="Cancel"
            icon="x"
            variant="ghost"
            tone="warning"
            loadingText="Cancelling"
            :requestStatus="cancelVotingRequestStatus"
            class="grow"
            @click="cancelVotingProcess"
          />
          <VButton
            :icon="votingKind === 'merge' ? 'tools' : 'trash'"
            :tone="votingKind === 'merge' ? 'success' : 'destructive'"
            loadingText="Applying Changes"
            :label="`Override Vote And ${votingKindCapitalized}`"
            :requestStatus="overrideButtonRequestStatus"
            :disabled="statusStoreUpdating"
            @click="completeVoting"
          />
        </div>
      </div>
    </Modal>

    <!-- only voters ever see this if the user who started voting leaves -->
    <Modal
      ref="userWhoStartedVotingLeftModalRef"
      title="User That Started Voting Has Left"
    >
      Voting on {{ votingKind === "merge" ? "applying" : "abandoning" }} this
      change set has been cancelled due to the user who started the vote not
      being online.
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { VButton, Modal, Icon } from "@si/vue-lib/design-system";
import clsx from "clsx";
import {
  PropType,
  computed,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import { useRoute } from "vue-router";
import { useToast } from "vue-toastification";
import UserIcon from "@/components/layout/navbar/UserIcon.vue";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import UserCard from "@/components/layout/navbar/UserCard.vue";
import ActionsList from "@/components/Actions/ActionsList.vue";
import { usePresenceStore, ONLINE_EXPIRATION } from "@/store/presence.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useStatusStore } from "@/store/status.store";
import { useActionsStore } from "@/store/actions.store";
import { useAuthStore } from "@/store/auth.store";
import ApprovalFlowCancelled from "@/components/toasts/ApprovalFlowCancelled.vue";

// If we end up adding another VotingKind, make sure to change ALL of the logic across this component that checks the votingKind prop!
export type VotingKind = "merge" | "abandon";

const toast = useToast();
const authStore = useAuthStore();
const actionsStore = useActionsStore();
const presenceStore = usePresenceStore();
const changeSetsStore = useChangeSetsStore();
const statusStore = useStatusStore();
const route = useRoute();

const votingModalRef = ref<InstanceType<typeof Modal> | null>(null);
const userWhoStartedVotingLeftModalRef = ref<InstanceType<typeof Modal> | null>(
  null,
);

const props = defineProps({
  votingKind: { type: String as PropType<VotingKind>, required: true },
});

const overrideButtonRequestStatus = changeSetsStore.getRequestStatus(
  props.votingKind === "merge" ? "APPLY_CHANGE_SET" : "ABANDON_CHANGE_SET",
);
const cancelVotingRequestStatus = changeSetsStore.getRequestStatus(
  props.votingKind === "merge"
    ? "CANCEL_APPROVAL_PROCESS"
    : "CANCEL_ABANDON_APPROVAL_PROCESS",
);

const votingKindCapitalized = computed(
  () => props.votingKind.charAt(0).toUpperCase() + props.votingKind.slice(1),
);
const usersInChangeSet = computed(() => presenceStore.usersInChangeSet);
const requiresVoting = computed(
  () => presenceStore.usersInChangeSet.length > 0,
);

const changeSet = computed(() => changeSetsStore.selectedChangeSet);
const statusStoreUpdating = computed(() => {
  if (statusStore.globalStatus) {
    return statusStore.globalStatus.isUpdating;
  } else return false;
});
const hasActions = computed(() => actionsStore.proposedActions.length > 0);
const beginApprovalReqStatus = changeSetsStore.getRequestStatus(
  props.votingKind === "merge"
    ? "BEGIN_APPROVAL_PROCESS"
    : "BEGIN_ABANDON_APPROVAL_PROCESS",
);
const doVotingActionReqStatus = changeSetsStore.getRequestStatus(
  props.votingKind === "merge" ? "APPLY_CHANGE_SET" : "ABANDON_CHANGE_SET",
);
const votingStartedByYou = computed(() => {
  if (props.votingKind === "merge") {
    return (
      changeSetsStore.selectedChangeSet?.mergeRequestedByUserId ===
      authStore.user?.pk
    );
  } else {
    return (
      changeSetsStore.selectedChangeSet?.abandonRequestedByUserId ===
      authStore.user?.pk
    );
  }
});
const votingRequestedByUserId = computed(() => {
  if (props.votingKind === "merge") {
    return changeSet.value?.mergeRequestedByUserId;
  } else {
    return changeSet.value?.abandonRequestedByUserId;
  }
});
const userWhoStartedVoting = computed(() => {
  const systemUser = {
    name: "unknown user",
    color: "magenta",
    status: "active",
  };
  if (votingStartedByYou.value && authStore.user) {
    return {
      name: authStore.user.name,
      pictureUrl: authStore.user.picture_url,
      color: "white",
      status: "active",
    };
  } else if (votingRequestedByUserId.value) {
    const user = presenceStore.usersById[votingRequestedByUserId.value];
    if (user) {
      return user;
    }

    return systemUser;
  }

  return systemUser;
});
const currentlyVoting = computed(() => {
  if (changeSetsStore.headSelected) return false;
  if (props.votingKind === "merge") {
    return changeSet.value?.status === ChangeSetStatus.NeedsApproval;
  } else {
    return changeSet.value?.status === ChangeSetStatus.NeedsAbandonApproval;
  }
});
const selectedChangeSetName = computed(
  () => changeSetsStore.selectedChangeSet?.name,
);

const rejectedWorkflow = ref<boolean>();
const successfullyVoted = ref<boolean>();
const allUsersVoted = ref<boolean>();
const canCloseModal = ref<boolean>(
  requiresVoting.value || changeSet.value?.status === ChangeSetStatus.Open,
);

function resetState() {
  rejectedWorkflow.value = false;
  successfullyVoted.value = false;
}

async function beginVoting() {
  canCloseModal.value = false;
  if (props.votingKind === "merge") {
    await changeSetsStore.BEGIN_APPROVAL_PROCESS();
  } else {
    await changeSetsStore.BEGIN_ABANDON_APPROVAL_PROCESS();
  }
}

async function completeVoting() {
  emit("completeVoting");
  votingModalRef.value?.close();
  clearInterval(cancelApplyIfOriginalUserGoesAway.value);
}

async function cancelVotingProcess() {
  emit("cancelVoting");
  if (props.votingKind === "merge") {
    await changeSetsStore.CANCEL_APPROVAL_PROCESS();
  } else {
    await changeSetsStore.CANCEL_ABANDON_APPROVAL_PROCESS();
  }
  rejectedWorkflow.value = true;
  canCloseModal.value = true;
  votingModalRef.value?.close();
}

async function vote(vote: string) {
  if (props.votingKind === "merge") {
    await changeSetsStore.APPLY_CHANGE_SET_VOTE(vote);
  } else {
    await changeSetsStore.APPLY_ABANDON_VOTE(vote);
  }
  successfullyVoted.value = true;
  canCloseModal.value = false;
  if (vote === "Reject") rejectedWorkflow.value = true;
  else rejectedWorkflow.value = false;
}

function openVotingModalHandler() {
  votingModalRef.value?.open();
  resetState();
  checkForUserWhoStartedVotingAndCancelIfTheyAreGone();
}

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
  if (
    (props.votingKind === "merge" &&
      changeSet.value?.status === ChangeSetStatus.NeedsApproval) ||
    (props.votingKind === "abandon" &&
      changeSet.value?.status === ChangeSetStatus.NeedsAbandonApproval)
  ) {
    openVotingModalHandler();
  }
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
  clearInterval(cancelApplyIfOriginalUserGoesAway.value);
});

const onKeyDown = async (e: KeyboardEvent) => {
  if (
    e.key === "Enter" &&
    votingModalRef.value?.isOpen &&
    changeSet.value?.status === ChangeSetStatus.Open
  ) {
    if (requiresVoting.value) {
      await beginVoting();
    } else {
      await completeVoting();
    }
  }
};

const cancelApplyIfOriginalUserGoesAway = ref<Timeout>();
const checkForUserWhoStartedVotingAndCancelIfTheyAreGone = () => {
  // If we start the app in a change set where we are not the userWhoStartedVoting, check to make sure the userWhoStartedVoting is present by waiting for an online ping
  if (!votingStartedByYou.value && currentlyVoting.value) {
    cancelApplyIfOriginalUserGoesAway.value = setInterval(() => {
      // After the length of the ONLINE_EXPIRATION plus one second, cancel the approval process if the userWhoStartedVoting is still not present
      if (
        userWhoStartedVoting.value.name === "unknown user" &&
        currentlyVoting.value
      ) {
        cancelVotingProcess();
        votingModalRef.value?.close();
        userWhoStartedVotingLeftModalRef.value?.open();
        clearInterval(cancelApplyIfOriginalUserGoesAway.value);
      }
    }, ONLINE_EXPIRATION + 1000);
  }
};

const checkVoting = () => {
  const votes = Object.values(changeSetsStore.changeSetApprovals);
  if (votes.length === 0) {
    // no votes have come in yet
    return;
  } else if (votes.indexOf("Reject") > -1) {
    // someone has voted reject
    rejectedWorkflow.value = true;
  }

  // We add one to the usersInChangeSet length because it doesn't include you!
  if (votes.length === usersInChangeSet.value.length + 1) {
    // everyone has voted!
    if (
      // everyone voted approve or pass
      _.every(votes, (a) => a === "Approve" || a === "Pass")
    ) {
      completeVoting();
    } else {
      // someone voted reject
      rejectedWorkflow.value = true;
      allUsersVoted.value = true;
    }
  }
};

// React to changes in the change set status - if we cancel or apply
watch(
  () => changeSetsStore.selectedChangeSet?.status,
  (newVal, oldVal) => {
    // this pops open the voting modal for the other users in the change set
    if (
      (props.votingKind === "merge" &&
        newVal === ChangeSetStatus.NeedsApproval) ||
      (props.votingKind === "abandon" &&
        newVal === ChangeSetStatus.NeedsAbandonApproval)
    ) {
      openVotingModalHandler();
      canCloseModal.value = false;
      successfullyVoted.value = false;
    }

    // this closes the voting modal when voting is cancelled
    if (
      newVal === ChangeSetStatus.Open &&
      ((oldVal === ChangeSetStatus.NeedsApproval &&
        props.votingKind === "merge") ||
        (oldVal === ChangeSetStatus.NeedsAbandonApproval &&
          props.votingKind === "abandon"))
    ) {
      votingModalRef.value?.close();
      canCloseModal.value = true;
      rejectedWorkflow.value = false;
      if (!changeSetsStore.headSelected) {
        toast({
          component: ApprovalFlowCancelled,
          props: {
            action: props.votingKind === "merge" ? "applying" : "abandoning",
          },
        });
      }
    }
  },
);

// Check the vote any time we receive a new vote in changeSetApprovals
watch(
  () => changeSetsStore.changeSetApprovals,
  () => {
    if (!votingStartedByYou.value) return;
    checkVoting();
  },
  {
    deep: true,
  },
);

// Check the vote any time the number of users in the change set changes
watch(
  () => presenceStore.usersInChangeSet,
  () => {
    if (!votingStartedByYou.value) return;
    checkVoting();
  },
);

const emit = defineEmits(["cancelVoting", "completeVoting"]);

defineExpose({ open: openVotingModalHandler });
</script>
