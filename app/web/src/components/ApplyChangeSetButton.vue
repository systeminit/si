<template>
  <VButton
    v-if="!changeSetsStore.headSelected"
    ref="applyButtonRef"
    icon="tools"
    size="md"
    tone="success"
    loadingText="Applying Changes"
    :requestStatus="applyChangeSetReqStatus"
    :disabled="statusStoreUpdating"
    @click.stop="openModalHandler"
  >
    Apply Changes

    <Modal
      ref="applyModalRef"
      title="Apply Change Set"
      :noExit="!canCloseModal"
    >
      <div class="max-h-[80vh] overflow-hidden flex flex-col">
        <template v-if="changeSet.status === ChangeSetStatus.NeedsApproval">
          <div :class="clsx('px-sm pb-sm pt-0 flex items-center gap-3')">
            <UserIcon :user="applyUser" />
            <div>
              <template v-if="appliedByYou">You have</template>
              <template v-else>
                <span class="italic">{{ applyUser?.name }}</span> has
              </template>
              clicked the Apply Changes button to apply all of the changes in
              this change set to Head.<template v-if="appliedByYou">
                There are other users online in this change set, so they will
                get the chance to cancel your apply.
              </template>
            </div>
          </div>
          <template v-if="appliedByYou">
            <div
              class="text-sm py-xs px-sm w-full bg-neutral-700 dark:text-neutral-200 text-white font-bold rounded-t-md"
            >
              <template v-if="!allUsersVoted"
                >{{ usersInChangeset.length }} users can approve</template
              >
            </div>
            <div
              class="overflow-y-auto flex-grow border dark:border-neutral-700 dark:bg-neutral-600 bg-neutral-200"
            >
              <div
                v-for="(user, index) in usersInChangeset"
                :key="index"
                class="flex items-center pr-sm justify-between gap-4"
              >
                <div class="min-w-0">
                  <UserCard :user="user" hideChangesetInfo hideStatus />
                </div>
                <span
                  v-if="!changeSetsStore.changeSetApprovals[user.pk]"
                  class="italic leading-tight"
                  >waiting...</span
                >
                <!-- while waiting show all options dimmer, once voted only show the selected vote -->
                <div class="grow flex justify-end">
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
                    class="text-neutral-400"
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
                        changeSetsStore.changeSetApprovals[user.pk] ===
                          'Reject',
                    }"
                  />
                </div>
              </div>
            </div>
            <div class="flex w-full justify-center pb-xs mt-sm gap-sm">
              <VButton
                tone="shade"
                variant="ghost"
                icon="x"
                label="Cancel"
                @click="cancelVoting"
              />
              <VButton
                icon="tools"
                size="sm"
                tone="success"
                loadingText="Applying Changes"
                label="Continue"
                :requestStatus="applyChangeSetReqStatus"
                :disabled="statusStoreUpdating"
                class="grow"
                @click="applyChangeSet"
              />
            </div>
          </template>
          <template v-else>
            <template v-if="!successfullyVoted">
              <div class="flex w-full justify-center pt-2 gap-2">
                <VButton
                  icon="thumbs-up"
                  variant="ghost"
                  tone="success"
                  loadingText="Approving"
                  label="Approve"
                  :disabled="statusStoreUpdating"
                  @click="changeSetApprovalVote('Approve')"
                />
                <VButton
                  icon="minus"
                  variant="ghost"
                  tone="error"
                  loadingText="Passing"
                  label="Pass"
                  :disabled="statusStoreUpdating"
                  @click="changeSetApprovalVote('Pass')"
                />
                <VButton
                  icon="thumbs-down"
                  variant="ghost"
                  tone="error"
                  loadingText="Rejecting"
                  label="Reject"
                  :disabled="statusStoreUpdating"
                  @click="changeSetApprovalVote('Reject')"
                />
              </div>
            </template>
            <template v-if="successfullyVoted">
              <div class="flex gap-4 w-full p-2">
                <Icon name="lock" size="lg" tone="warning" />
                <span class="text-sm align-middle"
                  >Changeset is locked until all users in the changeset have
                  voted on the merge</span
                >
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
                  class="border border-transparent dark:text-white hover:cursor-pointer hover:border-action-500 dark:hover:border-action-300 p-2"
                  >Go to head</RouterLink
                >
              </div>
            </template>
          </template>
        </template>
        <template v-else-if="changeSet.status === ChangeSetStatus.Open">
          <template v-if="!hasActions">
            <div class="text-center text-md mb-xs">
              Applying this change set may create, modify, or destroy real
              resources in the cloud.
            </div>
            <div class="text-center text-sm mb-sm">
              Are you sure you want to apply this change set?
            </div>
          </template>
          <template v-if="hasActions">
            <div class="text-center text-md mb-xs">
              Applying this change set may create, modify, or destroy real
              resources in the cloud.
            </div>
            <div class="text-center text-sm mb-sm">
              Pick which actions will be applied to the real world:
            </div>
            <div
              class="flex-grow overflow-y-auto mb-sm border border-neutral-100 dark:border-neutral-700"
            >
              <div
                class="flex flex-row justify-between place-items-center py-sm bg-neutral-100 dark:bg-neutral-700"
              >
                <div class="grow-0 mx-[.66em]">
                  <Icon
                    name="bullet-list"
                    class="attributes-panel-item__type-icon"
                    size="sm"
                  />
                </div>
                <div class="grow">
                  {{ actionsStore.proposedActions.length }} Total Action(s)
                </div>
                <div class="grow-0 flex flex-row mr-xs">
                  <div
                    v-for="(cnt, kind) in actionsStore.countActionsByKind"
                    :key="kind"
                    class="flex flex-row mx-2xs p-2xs rounded dark:bg-neutral-900 bg-neutral-200"
                  >
                    <div class="mx-2xs">{{ cnt }}</div>
                    <StatusIndicatorIcon
                      type="action"
                      :status="kind"
                      tone="shade"
                      size="sm"
                    />
                  </div>
                </div>
              </div>
              <ul>
                <li
                  v-for="(action, index) in actionsStore.proposedActions"
                  :key="action.id"
                  :class="
                    clsx(
                      'list-none',
                      index !== actionsStore.proposedActions.length - 1 &&
                        'border-b border-neutral-300 dark:border-neutral-700',
                    )
                  "
                >
                  <ActionCard
                    slim
                    :action="action"
                    @remove="actionsStore.REMOVE_ACTION(action.id)"
                  />
                </li>
              </ul>
            </div>
          </template>
          <div class="flex flex-row items-center w-full gap-sm">
            <VButton
              tone="shade"
              variant="ghost"
              icon="x"
              label="Cancel"
              @click="cancelApprovalProcess"
            />
            <VButton
              v-if="!changeSetsStore.headSelected"
              ref="applyButtonRef"
              class="flex-grow"
              icon="tools"
              tone="success"
              :loadingText="
                requiresVoting ? 'Beginning Approval Flow' : 'Applying Changes'
              "
              :label="requiresVoting ? 'Begin Approval Flow' : 'Apply Changes'"
              :requestStatus="
                requiresVoting
                  ? beginMergeApprovalReqStatus
                  : applyChangeSetReqStatus
              "
              :disabled="statusStoreUpdating"
              @click="requiresVoting ? beginMergeApproval() : applyChangeSet()"
            />
          </div>
        </template>
        <template v-if="rejectedWorkflow && appliedByYou">
          <div class="text-sm py-xs">
            One of the users in this changeset has rejected the merge. You can
            either override and merge the changeset, above or 'Cancel' the merge
            flow.
          </div>
          <VButton
            label="Cancel Merge Flow"
            variant="ghost"
            size="sm"
            tone="warning"
            loadingText="Cancelling Merge Flow"
            ruquestStatus="cancelMergeApprovalReqStatus"
            @click="cancelMergeHandler"
          />
        </template>
      </div>
    </Modal>

    <!-- only voters ever see this, not viewed by instigators -->
    <Modal ref="changeSetAppliedRef" title="Change Set Has Been Merged" noExit>
      <div
        class="bg-white dark:bg-neutral-700 rounded-lg flex flex-col items-center max-h-[90vh] shadow-md overflow-hidden pb-xs"
      >
        <div class="px-sm pt-sm pb-xs w-full">
          The change set you were in was merged by:
        </div>

        <div v-if="appliedByUser" class="pr-xs">
          <UserCard :user="appliedByUser" hideChangesetInfo hideStatus />
        </div>
        <div class="px-sm pb-sm pt-xs w-full">
          We are redirecting you to head. You can continue your work by creating
          a new change set or joining another existing change set.
        </div>
        <div class="self-stretch px-sm flex flex-row">
          <VButton
            class="flex-grow"
            label="Ok"
            variant="ghost"
            @click="changeSetAppliedHandler()"
          />
        </div>
      </div>
    </Modal>
  </VButton>
</template>

<script lang="ts" setup>
import { onMounted, computed, ref, watch, onBeforeUnmount } from "vue";
import * as _ from "lodash-es";
import { useRouter, useRoute } from "vue-router";
import { VButton, Modal, Icon } from "@si/vue-lib/design-system";
import JSConfetti from "js-confetti";
import clsx from "clsx";
import { storeToRefs } from "pinia";
import ActionCard from "@/components/ActionCard.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useStatusStore } from "@/store/status.store";
import { useActionsStore } from "@/store/actions.store";
import { usePresenceStore } from "@/store/presence.store";
import UserIcon from "@/components/layout/navbar/UserIcon.vue";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { useAuthStore } from "@/store/auth.store";
import UserCard from "@/components/layout/navbar/UserCard.vue";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";

const applyModalRef = ref<InstanceType<typeof Modal> | null>(null);
const presenceStore = usePresenceStore();
const statusStore = useStatusStore();
const changeSetsStore = useChangeSetsStore();
const actionsStore = useActionsStore();
const authStore = useAuthStore();
const router = useRouter();
const route = useRoute();

const usersInChangeset = computed(() => presenceStore.usersInChangeset);

// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
const changeSet = computed(() => changeSetsStore.selectedChangeSet!);

const hasActions = computed(() => actionsStore.proposedActions.length > 0);
const requiresVoting = computed(
  () => presenceStore.usersInChangeset.length > 0,
);

const changeSetAppliedRef = ref();
// TODO: make these all computed
const rejectedWorkflow = ref<boolean>();
const successfullyVoted = ref<boolean>();
const allUsersVoted = ref<boolean>();
const canCloseModal = ref<boolean>(
  requiresVoting.value || changeSet.value.status === ChangeSetStatus.Open,
);

function openModalHandler() {
  changeSetAppliedRef.value?.close();
  applyModalRef.value?.open();
  resetState();
}

function resetState() {
  rejectedWorkflow.value = false;
  successfullyVoted.value = false;
}

let jsConfetti: JSConfetti;
const confettis = [
  { emojis: ["🎉"] },
  { emojis: ["🍿"] },
  { emojis: ["🤘", "🤘🏻", "🤘🏼", "🤘🏽", "🤘🏾", "🤘🏿"] },
  { emojis: ["❤️", "🧡", "💛", "💚", "💙", "💜"] },
  { emojis: ["🍾", "🍷", "🍸", "🍹", "🍺", "🥂", "🍻"] },
  { emojis: ["🏳️‍🌈", "🏳️‍⚧️", "⚡️", "🌈", "✨", "🔥", "🇧🇷"] },
];
onMounted(() => {
  jsConfetti = new JSConfetti({
    canvas:
      (document.getElementById("confetti") as HTMLCanvasElement) || undefined,
  });
  window.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});

const onKeyDown = async (e: KeyboardEvent) => {
  if (
    e.key === "Enter" &&
    applyModalRef.value?.isOpen &&
    changeSet.value.status === ChangeSetStatus.Open
  ) {
    if (requiresVoting.value) {
      await beginMergeApproval();
    } else {
      await applyChangeSet();
    }
  }
};

const applyChangeSetReqStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET");
// Applies the current change set3
const applyChangeSet = async () => {
  if (!route.name) return;
  applyModalRef.value?.close();
  await changeSetsStore.APPLY_CHANGE_SET(authStore.user?.email ?? "");
  router.replace({
    name: route.name,
    params: {
      ...route.params,
      changeSetId: "head",
    },
  });
  // TODO this will fire after actions have run
  await jsConfetti.addConfetti(_.sample(confettis));
};

const beginMergeApprovalReqStatus = changeSetsStore.getRequestStatus(
  "BEGIN_APPROVAL_PROCESS",
);
const beginMergeApproval = async () => {
  await changeSetsStore.BEGIN_APPROVAL_PROCESS();
};

async function changeSetApprovalVote(vote: string) {
  await changeSetsStore.APPLY_CHANGE_SET_VOTE(vote);
  successfullyVoted.value = true;
  canCloseModal.value = false;
  rejectedWorkflow.value = false;
}

async function cancelMergeHandler() {
  await cancelApprovalProcess();
}

async function cancelVoting() {
  await cancelApprovalProcess();
}

async function cancelApprovalProcess() {
  await changeSetsStore.CANCEL_APPROVAL_PROCESS();
  rejectedWorkflow.value = true;
  canCloseModal.value = true;
  applyModalRef.value?.close();
}

const appliedByYou = computed(
  () =>
    changeSetsStore.selectedChangeSet?.mergeRequestedByUserId ===
    authStore.user?.pk,
);
const applyUser = computed(() => {
  const systemUser = {
    name: "System User",
    color: "magenta",
    status: "active",
  };
  if (appliedByYou.value && authStore.user) {
    return {
      name: authStore.user.name,
      pictureUrl: authStore.user.picture_url,
      color: "white",
      status: "active",
    };
  } else if (changeSet.value?.mergeRequestedByUserId) {
    const user =
      presenceStore.usersById[changeSet.value?.mergeRequestedByUserId];
    if (user) {
      return user;
    }

    return systemUser;
  }

  return systemUser;
});

const statusStoreUpdating = computed(() => {
  if (statusStore.globalStatus) {
    return statusStore.globalStatus.isUpdating;
  } else return false;
});

watch(
  () => changeSetsStore.selectedChangeSet?.status,
  (newVal, oldVal) => {
    if (newVal === ChangeSetStatus.NeedsApproval) {
      applyModalRef.value?.open();
      successfullyVoted.value = false;
    }
    if (
      newVal === ChangeSetStatus.Open &&
      oldVal === ChangeSetStatus.NeedsApproval
    ) {
      applyModalRef.value?.close();
      canCloseModal.value = true;
      rejectedWorkflow.value = false;
    }

    // this moves voters from the changeset to head
    if (
      newVal === ChangeSetStatus.Applied &&
      oldVal === ChangeSetStatus.NeedsApproval
    ) {
      applyModalRef.value?.close();
      changeSetAppliedRef.value?.open();
    }
    // NOTE, when there are no votes required newVal = Applied && oldVal = Open
  },
);

/* FUTURE: we have a hole here (we need storage on the backend to resolve)
Person 1 & 2 are on a changeset
Person 1 starts the vote
Person 3 joins the changeset
  they missed the "please vote WS Event"
We now expect 3 votes
Person 1 will be forced to hit override to move forward

We can now look at "are you looking at a changeset that needs approval?"
And show them the vote prompt
*/
watch(
  () => changeSetsStore.changeSetApprovals,
  () => {
    if (!appliedByYou.value) return;
    if (
      _.values(changeSetsStore.changeSetApprovals).length !==
      usersInChangeset.value.length // This is the number of other users (without the person who triggered the merge)
    )
      return;
    if (
      _.every(
        _.values(changeSetsStore.changeSetApprovals),
        (a) => a === "Approve" || a === "Pass",
      )
    ) {
      applyChangeSet();
    } else {
      rejectedWorkflow.value = true;
      allUsersVoted.value = true;
    }
  },
  {
    deep: true,
  },
);

const { postApplyActor } = storeToRefs(changeSetsStore);
watch(postApplyActor, () => {
  if (
    postApplyActor.value !== null &&
    postApplyActor.value !== authStore.user?.pk
  ) {
    changeSetAppliedRef.value?.open();
  }
});

function changeSetAppliedHandler() {
  changeSetAppliedRef.value?.close();
  // Redirect the user to head changeset
  if (route.name) {
    router.push({
      name: route.name,
      params: {
        ...route.params,
        changeSetId: "head",
      },
    });
  }
}

const appliedByUser = computed(() => {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return presenceStore.usersById[changeSetsStore.postApplyActor!];
});
</script>
