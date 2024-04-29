<template>
  <div>
    <div class="flex gap-xs items-end">
      <label>
        <div
          class="text-[11px] mt-[1px] mb-[5px] capsize font-medium text-neutral-300"
        >
          CHANGE&nbsp;SET:
        </div>
        <VormInput
          ref="dropdownRef"
          class="flex-grow font-bold mb-[-1px]"
          size="xs"
          type="dropdown"
          noLabel
          placeholder="-- select a change set --"
          :modelValue="selectedChangeSetId"
          :options="changeSetDropdownOptions"
          @update:model-value="onSelectChangeSet"
        />
      </label>

      <VButton
        v-tooltip="{
          content: 'Create Change Set',
        }"
        tone="action"
        variant="ghost"
        icon="git-branch-plus"
        size="sm"
        :disabled="
          actionsStore.actionsAreInProgress &&
          featureFlagStore.DONT_BLOCK_ON_ACTIONS
        "
        @click="openCreateModal"
      />

      <VButton
        v-tooltip="{
          content: 'Abandon Change Set',
        }"
        tone="action"
        variant="ghost"
        icon="trash"
        size="sm"
        :disabled="
          (actionsStore.actionsAreInProgress &&
            featureFlagStore.DONT_BLOCK_ON_ACTIONS) ||
          !selectedChangeSetName ||
          changeSetsStore.headSelected
        "
        @click="abandonConfirmationModalRef.open()"
      />
    </div>

    <Modal ref="createModalRef" title="Create Change Set">
      <form @submit.prevent="onCreateChangeSet">
        <Stack>
          <p>
            Modeling a configuration or extending SI happens within
            <b>Change Sets</b>. Think of these like light-weight branches,
            allowing you to experiment freely without risk of impacting
            production systems.
          </p>
          <p>
            Please give your <b>Change Set</b> a name below, and click the
            Create button.
          </p>
          <VormInput
            v-model="createChangeSetName"
            label="Change set name"
            required
            :regex="CHANGE_SET_NAME_REGEX"
            regexMessage="You cannot name a change set 'HEAD' - please choose another name."
            requiredMessage="Please choose a name for your change set!"
          />
          <div class="flex flex-row-reverse gap-sm">
            <VButton
              :disabled="validationState.isError"
              tone="success"
              icon="plus-circle"
              label="Create change set"
              loadingText="Creating Change Set"
              :requestStatus="createChangeSetReqStatus"
              class="flex-grow"
              submit
            />
          </div>
        </Stack>
      </form>
    </Modal>

    <Modal ref="abandonConfirmationModalRef" title="Abandon Change Set?">
      <div
        v-if="changeSet?.status === ChangeSetStatus.NeedsAbandonApproval"
        class="max-h-[80vh] overflow-hidden flex flex-col"
      >
        <div :class="clsx('px-sm pb-sm pt-0 flex items-center gap-3')">
          <UserIcon :user="abandonUser" />
          <div>
            <template v-if="abandonedByYou">You have</template>
            <template v-else>
              <span class="italic">{{ abandonUser?.name }}</span> has
            </template>
            clicked the Abandon Change Set button to abandon this change
            set.<template v-if="abandonedByYou">
              There are other users online in this change set, so they will get
              the chance to cancel your apply.
            </template>
          </div>
        </div>
        <template v-if="abandonedByYou">
          <div class="flex w-full justify-center pb-xs">
            <VButton
              icon="tools"
              size="sm"
              tone="success"
              loadingText="Abandoning Change Set"
              label="Abandon Change Set"
              :requestStatus="abandonChangeSetReqStatus"
              :disabled="statusStoreUpdating"
              @click="overrideAbandonChangesetHandler"
            />
          </div>
          <div class="text-sm pb-2 italic text-center w-full text-neutral-400">
            <template v-if="!allUsersVoted"
              >Waiting on other users in the changeset to vote.</template
            >
          </div>
          <div
            class="overflow-y-auto flex-grow border border-neutral-300 dark:border-neutral-700"
          >
            <div
              v-for="(user, index) in usersInChangeset"
              :key="index"
              class="flex items-center pr-sm justify-center gap-4"
            >
              <div class="min-w-0">
                <UserCard :user="user" hideChangesetInfo />
              </div>
              <Icon
                v-if="changeSetsStore.changeSetApprovals[user.pk] === 'Approve'"
                name="thumbs-up"
                size="lg"
                class="text-success-400"
              />
              <Icon
                v-else-if="
                  changeSetsStore.changeSetApprovals[user.pk] === 'Pass'
                "
                name="minus"
                size="lg"
                class="text-neutral-400"
              />
              <Icon
                v-else-if="
                  changeSetsStore.changeSetApprovals[user.pk] === 'Reject'
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
                label="Approve"
                :disabled="statusStoreUpdating"
                @click="changeSetAbandonVote('Approve')"
              />
              <VButton
                icon="minus"
                variant="ghost"
                tone="error"
                loadingText="Passing"
                label="Pass"
                :disabled="statusStoreUpdating"
                @click="changeSetAbandonVote('Pass')"
              />
              <VButton
                icon="thumbs-down"
                variant="ghost"
                tone="error"
                loadingText="Rejecting"
                label="Reject"
                :disabled="statusStoreUpdating"
                @click="changeSetAbandonVote('Reject')"
              />
            </div>
          </template>
          <template v-if="successfullyVoted">
            <div class="flex gap-4 w-full p-xs">
              <Icon name="lock" size="lg" tone="warning" />
              <span class="text-sm align-middle">
                Changeset is locked until all users in the changeset have voted
                on the abandonment.
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
                >Go to head</RouterLink
              >
            </div>
          </template>
        </template>
      </div>
      <Stack
        v-else-if="changeSet?.status === ChangeSetStatus.Open && !override"
      >
        <p>
          Are you sure that you want to abandon change set
          <span class="italic font-bold">"{{ selectedChangeSetName }}"</span>
          and return to HEAD?
        </p>
        <p>Once abandoned, a change set cannot be recovered.</p>
        <Inline fullWidth>
          <VButton
            label="Cancel"
            tone="shade"
            variant="ghost"
            icon="x"
            @click="abandonConfirmationModalRef.close"
          />
          <VButton
            :label="`Yes - ${
              abandonRequiresVoting
                ? 'Begin Approval Flow'
                : 'Abandon Change Set'
            }`"
            tone="destructive"
            icon="trash"
            :requestStatus="abandonChangeSetReqStatus"
            loadingText="Abandoning Changeset..."
            @click="
              () => {
                abandonRequiresVoting
                  ? abandonChangesetApproval()
                  : abandonChangeset();
              }
            "
          />
        </Inline>
      </Stack>
    </Modal>

    <Modal
      ref="changeSetAbandonedRef"
      title="Change Set Has Been Abandoned"
      noExit
    >
      <div
        class="bg-white dark:bg-neutral-700 rounded-lg flex flex-col items-center max-h-[90vh] shadow-md overflow-hidden pb-xs"
      >
        <div class="px-sm pt-sm pb-xs w-full">
          The change set you were in was abandoned by:
        </div>

        <div v-if="abandonedByUser" class="pr-xs">
          <UserCard :user="abandonedByUser" hideChangesetInfo />
        </div>
        <div class="px-sm pb-sm pt-xs w-full">
          You are now on Head. You can continue your work by creating a new
          change set or joining another existing change set.
        </div>
        <div class="self-stretch px-sm flex flex-row">
          <VButton
            class="flex-grow"
            label="Ok"
            variant="ghost"
            @click="changeSetAbandonedHandler()"
          />
        </div>
      </div>
    </Modal>

    <Wipe ref="wipeRef">
      <template #duringWipe>
        <VButton
          icon="loader"
          size="md"
          label="Merging"
          class="!bg-action-600"
        />
      </template>
      <template #afterWipe>
        <div
          v-if="changeSetMergeStatus.isPending || wipeRef?.state === 'running'"
          class="gap-xs items-center flex flex-row p-xl min-w-0 w-full justify-center"
        >
          <Icon name="loader" size="2xl" />
          <span class="text-3xl italic truncate">
            Merging Change Set<template v-if="selectedChangeSetName">
              "{{ selectedChangeSetName }}"
            </template>
          </span>
        </div>
      </template>
    </Wipe>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { onMounted, onBeforeUnmount, computed, ref, watch } from "vue";
import * as _ from "lodash-es";
import { useRoute, useRouter } from "vue-router";
import {
  VButton,
  Icon,
  VormInput,
  Stack,
  Modal,
  useValidatedInputGroup,
  Inline,
} from "@si/vue-lib/design-system";
import { storeToRefs } from "pinia";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useActionsStore } from "@/store/actions.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { usePresenceStore } from "@/store/presence.store";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { useAuthStore } from "@/store/auth.store";
import { useStatusStore } from "@/store/status.store";
import Wipe from "../../Wipe.vue";
import UserIcon from "./UserIcon.vue";
import UserCard from "./UserCard.vue";

const CHANGE_SET_NAME_REGEX = /^(?!head).*$/i;

const dropdownRef = ref();
const abandonConfirmationModalRef = ref();
const changeSetAbandonedRef = ref();
const wipeRef = ref<InstanceType<typeof Wipe>>();

const statusStore = useStatusStore();
const authStore = useAuthStore();
const presenceStore = usePresenceStore();
const changeSetsStore = useChangeSetsStore();
const featureFlagStore = useFeatureFlagsStore();
const actionsStore = useActionsStore();
const openChangeSets = computed(() => changeSetsStore.openChangeSets);
const selectedChangeSetId = computed(() => changeSetsStore.selectedChangeSetId);
const selectedChangeSetName = computed(
  () => changeSetsStore.selectedChangeSet?.name,
);

const changeSetDropdownOptions = computed(() => {
  return [
    ..._.map(openChangeSets.value, (cs) => ({ value: cs.id, label: cs.name })),
    { value: "NEW", label: "+ Create new change set" },
  ];
});

const usersInChangeset = computed(() => presenceStore.usersInChangeset);

const router = useRouter();
const route = useRoute();

const createModalRef = ref<InstanceType<typeof Modal>>();

const changeSetsReqStatus =
  changeSetsStore.getRequestStatus("FETCH_CHANGE_SETS");

const checkFirstLoad = () => {
  if (!changeSetsReqStatus.value.isSuccess || !createModalRef.value) return;

  const isFirstLoad = !window.localStorage.getItem("ran-first-load");
  window.localStorage.setItem("ran-first-load", "true");

  if (isFirstLoad) {
    createModalRef.value?.open();
  }
};

watch([changeSetsReqStatus], checkFirstLoad);

onMounted(() => {
  checkFirstLoad();
  window.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});

const onKeyDown = async (e: KeyboardEvent) => {
  if (e.key === "Enter" && abandonConfirmationModalRef.value?.isOpen) {
    abandonChangeset();
  }
};

// The name for a new change set
const createChangeSetName = ref(changeSetsStore.getGeneratedChangesetName());

const { validationState, validationMethods } = useValidatedInputGroup();

function onSelectChangeSet(newVal: string) {
  if (newVal === "NEW") {
    createModalRef.value?.open();
    return;
  }

  if (newVal && route.name) {
    // do not allow people to navigate to a changeset that NeedsApproval
    // unless they were the one that initiated the merge request (avoids dead end)
    if (
      changeSetsStore.changeSetsById[newVal]?.status !== ChangeSetStatus.Open &&
      changeSetsStore.changeSetsById[newVal]?.mergeRequestedByUserId !==
        authStore.user?.pk
    ) {
      return;
    }

    // keep everything in the current route except the change set id
    // note - we use push here, so there is a new browser history entry
    router.push({
      name: route.name,
      params: {
        ...route.params,
        changeSetId: newVal,
      },
      query: route.query,
    });
  }
}

async function onCreateChangeSet() {
  if (validationMethods.hasError()) return;

  const createReq = await changeSetsStore.CREATE_CHANGE_SET(
    createChangeSetName.value,
  );
  createChangeSetName.value = changeSetsStore.getGeneratedChangesetName();

  if (createReq.result.success) {
    // reusing above to navigate to new change set... will probably clean this all up later
    onSelectChangeSet(createReq.result.data.changeSet.id);
    createModalRef.value?.close();
  }
}

const createChangeSetReqStatus =
  changeSetsStore.getRequestStatus("CREATE_CHANGE_SET");

const changeSetMergeStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET");

function openCreateModal() {
  createChangeSetName.value = changeSetsStore.getGeneratedChangesetName();
  createModalRef.value?.open();
}

const abandonChangeSetReqStatus =
  changeSetsStore.getRequestStatus("ABANDON_CHANGE_SET");

const override = ref(false);

function overrideAbandonChangesetHandler() {
  override.value = true;
  abandonChangeset();
}

async function abandonChangeset() {
  await changeSetsStore.ABANDON_CHANGE_SET();

  abandonConfirmationModalRef.value.close();

  if (route.name) {
    router.push({
      name: route.name,
      params: {
        ...route.params,
        changeSetId: "head",
      },
    });
  }

  await changeSetsStore.FETCH_CHANGE_SETS();

  // TODO(Wendy) - a temporary fix until we figure out and fix the bug where components from the abandoned changeset do not disappear from the diagram
  // eslint-disable-next-line no-restricted-globals
  location.reload();
}

const abandonRequiresVoting = computed(
  () => presenceStore.usersInChangeset.length > 0,
);

const changeSet = computed(() => changeSetsStore.selectedChangeSet);

const rejectedWorkflow = ref<boolean>();
const successfullyVoted = ref<boolean>();
const allUsersVoted = ref<boolean>();
const canCloseModal = ref<boolean>(
  !changeSet.value ||
    abandonRequiresVoting.value ||
    changeSet.value.status === ChangeSetStatus.Open,
);

const abandonedByYou = computed(
  () =>
    changeSetsStore.selectedChangeSet?.abandonRequestedByUserId ===
    authStore.user?.pk,
);

const abandonUser = computed(() => {
  const systemUser = {
    name: "System User",
    color: "magenta",
    status: "active",
  };
  if (abandonedByYou.value && authStore.user) {
    return {
      name: authStore.user.name,
      pictureUrl: authStore.user.picture_url,
      color: "white",
      status: "active",
    };
  } else if (changeSet.value?.abandonRequestedByUserId) {
    const user =
      presenceStore.usersById[changeSet.value?.abandonRequestedByUserId];
    if (user) {
      return user;
    }

    return systemUser;
  }

  return systemUser;
});

const statusStoreUpdating = computed(
  () => statusStore.globalStatus?.isUpdating ?? false,
);

const abandonChangesetApproval = async () => {
  await changeSetsStore.BEGIN_ABANDON_APPROVAL_PROCESS();
};

async function changeSetAbandonVote(vote: string) {
  await changeSetsStore.APPLY_ABANDON_VOTE(vote);
  successfullyVoted.value = true;
  canCloseModal.value = false;
  rejectedWorkflow.value = false;
}

function changeSetAbandonedHandler() {
  changeSetAbandonedRef.value?.close();
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

const abandonedByUser = computed(() => {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return presenceStore.usersById[changeSetsStore.postAbandonActor!];
});

watch(
  () => changeSetsStore.selectedChangeSet?.status,
  (newVal, oldVal) => {
    if (newVal === ChangeSetStatus.NeedsAbandonApproval) {
      abandonConfirmationModalRef.value?.open();
      successfullyVoted.value = false;
    }
    if (
      newVal === ChangeSetStatus.Open &&
      oldVal === ChangeSetStatus.NeedsAbandonApproval
    ) {
      abandonConfirmationModalRef.value?.close();
      canCloseModal.value = true;
      rejectedWorkflow.value = false;
    }

    if (
      newVal === ChangeSetStatus.Abandoned &&
      oldVal === ChangeSetStatus.NeedsAbandonApproval
    ) {
      abandonConfirmationModalRef.value?.close();
      // TODO(Wendy) - modal for users to tell them their change set was abandoned
      // changeSetAppliedRef.value?.open();
    }
  },
);

watch(
  () => changeSetsStore.changeSetApprovals,
  () => {
    if (!abandonedByYou.value) return;
    if (
      _.values(changeSetsStore.changeSetApprovals).length !==
      usersInChangeset.value.length + 1
      // This is the number of other users + the person who triggered the merge
    )
      return;
    if (
      _.every(
        _.values(changeSetsStore.changeSetApprovals),
        (a) => a === "Approve" || a === "Pass",
      )
    ) {
      abandonChangeset();
    } else {
      rejectedWorkflow.value = true;
      allUsersVoted.value = true;
    }
  },
  {
    deep: true,
  },
);

const { postAbandonActor } = storeToRefs(changeSetsStore);
watch(postAbandonActor, () => {
  if (
    postAbandonActor.value !== null &&
    postAbandonActor.value !== authStore.user?.pk
  ) {
    changeSetAbandonedRef.value?.open();
  }
});
</script>
