<template>
  <Popover ref="internalRef" popDown onTopOfEverything :noExit="!cancelled">
    <Teleport to="body">
      <TransitionRoot
        :show="internalRef.isOpen && !cancelled"
        appear
        as="template"
      >
        <TransitionChild
          as="template"
          enter="duration-300 ease-out"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="duration-200 ease-in"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <div class="fixed inset-0 bg-shade-100 bg-opacity-60 z-90" />
        </TransitionChild>
      </TransitionRoot>
    </Teleport>
    <div
      class="absolute top-0 left-[50%] translate-x-[-50%] translate-y-[-100%] w-0 h-0 border-transparent border-b-white dark:border-b-neutral-700 border-[16px] border-b-[10px]"
    />
    <div
      class="bg-white dark:bg-neutral-700 rounded-lg flex flex-col w-96 max-h-[90vh] shadow-md overflow-hidden"
    >
      <template v-if="cancelled">
        <div class="p-sm flex flex-col items-center gap-xs">
          <template v-if="cancelUser">
            <div :class="clsx('w-full', cancelUser.name === 'You' && 'pb-xs')">
              <span :class="cancelUser.name !== 'You' && 'italic'">{{
                cancelUser.name
              }}</span>
              cancelled applying this change set.
            </div>

            <div v-if="cancelUser.name !== 'You'" class="pr-xs">
              <UserCard :user="cancelUser" />
            </div>
          </template>
          <template v-else>The change set apply was cancelled.</template>
          <VButton
            label="Dismiss"
            variant="ghost"
            size="sm"
            @click="internalRef.close"
          />
        </div>
      </template>
      <template v-else>
        <div
          :class="
            clsx(
              'p-sm flex flex-row gap-xs items-center',
              !appliedByYou && 'border-b dark:border-neutral-500',
            )
          "
        >
          <UserIcon :user="applyUser" />
          <div>
            <template v-if="appliedByYou">You have</template>
            <template v-else>
              <span class="italic">{{ applyUser.name }}</span> has
            </template>
            clicked the Apply Changes button to apply all of the changes in this
            change set to Head.<template v-if="appliedByYou">
              There are other users online in this change set, so they will get
              the chance to cancel your apply.
            </template>
          </div>
        </div>
        <div
          v-if="appliedByYou"
          class="w-full flex flex-col items-center border-b dark:border-neutral-500 pb-xs"
        >
          <VButton label="Skip Approval And Apply" tone="warning" size="sm" />
        </div>
        <div
          v-if="!appliedByYou"
          class="p-sm border-b dark:border-neutral-500 flex flex-row items-center gap-xs justify-between"
        >
          <div class="flex flex-row gap-xs items-center">
            <UserIcon :user="presenceStore.myUserInfo" />
            <div class="flex flex-col">
              <div>Apply this change set?</div>
              <div class="text-xs text-neutral-400 pt-[2px]">
                <template v-if="yourVote">
                  <template v-if="yourVote === 'Deny'">Cancelling...</template>
                  <template v-else>Waiting for other users to vote...</template>
                </template>
                <template v-else
                  >You have {{ voteTimer }} seconds to vote.</template
                >
              </div>
            </div>
          </div>
          <div v-if="yourVote">
            <Icon
              v-if="yourVote === 'Approve'"
              name="thumbs-up"
              size="lg"
              class="text-success-400"
            />
            <Icon
              v-if="yourVote === 'Deny'"
              name="thumbs-down"
              size="lg"
              class="text-destructive-500"
            />
            <Icon
              v-if="yourVote === 'Abstain'"
              name="minus"
              size="lg"
              class="text-neutral-300"
            />
          </div>
          <div
            v-else
            class="flex flex-row gap-xs items-center text-neutral-300"
          >
            <div
              class="hover:text-success-400 cursor-pointer"
              @click="voteAccept"
            >
              <Icon name="thumbs-up" size="lg" />
            </div>
            <div
              class="hover:text-destructive-500 cursor-pointer"
              @click="voteCancel"
            >
              <Icon name="thumbs-down" size="lg" />
            </div>
            <VButton label="Pass" variant="ghost" size="xs" @click="votePass" />
          </div>
        </div>

        <div
          class="text-sm italic text-center w-full text-neutral-400 border-b dark:border-neutral-500"
        >
          Other users in this change set
          <template v-if="appliedByYou"
            >have {{ voteTimer }} seconds to vote.</template
          >
        </div>
        <div class="overflow-y-auto">
          <div
            v-for="(user, index) in votingUsers"
            :key="index"
            class="flex flex-row items-center overflow-hidden pr-sm justify-between"
          >
            <template v-if="user.vote !== 'Abstain'">
              <div class="min-w-0">
                <UserCard :user="user" hideChangesetInfo />
              </div>
              <Icon
                v-if="user?.vote === 'Approve'"
                name="thumbs-up"
                size="lg"
                class="text-success-400"
              />
              <Icon
                v-else-if="user?.vote === 'Deny'"
                name="thumbs-down"
                size="lg"
                class="text-destructive-500"
              />
            </template>
          </div>
        </div>
      </template>
    </div>
  </Popover>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { Icon, VButton } from "@si/vue-lib/design-system";
import * as _ from "lodash-es";
import { TransitionChild, TransitionRoot } from "@headlessui/vue";
import clsx from "clsx";
import Popover from "@/components/Popover.vue";
import { useAuthStore } from "@/store/auth.store";
import { OnlineUserInfo, usePresenceStore } from "@/store/presence.store";
import UserCard from "./UserCard.vue";
import UserIcon from "./UserIcon.vue";

export type ChangeSetVote = "Approve" | "Deny" | "Abstain" | undefined;

const VOTE_TIME = 30;
const authStore = useAuthStore();
const internalRef = ref();

const presenceStore = usePresenceStore();

const props = defineProps({
  appliedByYou: { type: Boolean },
});

// TODO(Wendy) - Mock data, should populate with users in the current changeset
const applyUser = ref<OnlineUserInfo>({
  pk: "xyz",
  name: "cool user 666",
  color: "magenta",
  idle: false,
  changeSetId: "123",
  pictureUrl: "https://placekitten.com/50/50",
});

const votingUsers = [
  {
    pk: "u1",
    name: "test user 1",
    color: "red",
    idle: false,
  },
  {
    pk: "u2",
    name: "test user 2 has an extremely long name woah dang so long",
    color: "green",
    idle: false,
    vote: "Approve" as ChangeSetVote,
  },
  {
    pk: "u3",
    name: "test user 3",
    color: "blue",
    idle: false,
    vote: "Deny" as ChangeSetVote,
  },
  {
    pk: "u4",
    name: "test user 4",
    color: "yellow",
    idle: false,
    vote: "Abstain" as ChangeSetVote,
  },
  {
    pk: "u5",
    name: "test user 5",
    color: "cyan",
    idle: false,
  },
];
// End mock data

const yourVote = ref();
const cancelled = ref(false);
const cancelUser = ref();
const closeTimeout = ref();
const voteTimer = ref();
const voteTimerTimeout = ref();

const startVoting = () => {
  if (props.appliedByYou) {
    applyUser.value.name = "You";
    applyUser.value.pictureUrl = authStore.user?.picture_url ?? null;
  }
  if (closeTimeout.value) clearTimeout(closeTimeout.value);
  closeTimeout.value = undefined;
  cancelled.value = false;
  cancelUser.value = undefined;
  yourVote.value = undefined;
  voteTimer.value = VOTE_TIME;
  voteTimerTimeout.value = setInterval(() => {
    voteTimer.value--;
    if (voteTimer.value < 1) {
      clearTimeout(voteTimerTimeout.value);
      if (yourVote.value) return;
      yourVote.value = "Abstain";
    }
  }, 1000);
};

const voteAccept = () => {
  // TODO(Wendy) - process an accept vote
  yourVote.value = "Approve";
};

const voteCancel = () => {
  // TODO(Wendy) - process a cancel vote, for now this just transitions the Popover
  yourVote.value = "Deny";

  // Slight delay before moving on to the cancel screen
  setTimeout(() => {
    cancelled.value = true;
    cancelUser.value = {
      name: "You",
      pictureUrl: authStore.user?.picture_url ?? null,
    };
  }, 500);
  // 10 seconds after cancelling the Popover should close
  closeTimeout.value = setTimeout(close, 10000);
};

const votePass = () => {
  // TODO(Wendy) - process a pass vote
  yourVote.value = "Abstain";
};

const open = (e?: MouseEvent, anchorToMouse?: boolean) => {
  startVoting();
  internalRef.value.open(e, anchorToMouse);
};

const openAt = (pos: { x: number; y: number }) => {
  startVoting();
  internalRef.value.openAt(pos);
};

const close = () => {
  internalRef.value.close();
  if (closeTimeout.value) clearTimeout(closeTimeout.value);
  closeTimeout.value = undefined;
};

const isOpen = computed(() => internalRef.value.isOpen);

defineExpose({
  open,
  openAt,
  close,
  isOpen,
});
</script>
