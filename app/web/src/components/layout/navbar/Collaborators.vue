<template>
  <div
    :class="
      clsx(
        'flex flex-row justify-evenly items-center m-xs',
        width,
        moreUsersPopoverRef?.isOpen && 'pointer-events-none',
      )
    "
  >
    <!-- Displays all visible users, up to 6 of them -->
    <template v-if="!(showOneIcon && displayUsers.length > 1)">
      <div v-for="(user, index) in displayUsers" :key="index" class="h-8 w-0">
        <UserIcon
          :tooltip="userTooltips[index]"
          :user="user"
          class="absolute translate-x-[-50%]"
          hasHoverState
          forceDark
        />
      </div>
    </template>

    <!-- If there are 7 or more users or if the screen is small and there are multiple users, some are put into this menu -->
    <div
      v-if="
        sortedUsers.length !== 1 &&
        (showOneIcon || sortedUsers.length > 6) &&
        sortedUsers.length > 0
      "
      class="h-8 w-0"
    >
      <div
        ref="moreUsersButtonRef"
        v-tooltip="moreUsersTooltip"
        :class="
          clsx(
            'absolute translate-x-[-50%] h-8 w-8 cursor-pointer bg-black',
            'border-2 border-shade-0 rounded-full overflow-hidden',
            'flex flex-row items-center hover:border-action-300',
            moreUsersNumber < 10
              ? 'text-base'
              : moreUsersNumber < 100
              ? 'text-xs'
              : 'text-xl',
          )
        "
        @click="openMoreUsersPopover"
      >
        <div class="text-center w-full font-bold">
          <Icon v-if="showOneIcon" name="user-circle" size="full" />
          <template v-else-if="moreUsersNumber < 100"
            >+{{ moreUsersNumber }}</template
          >
          <template v-else>+</template>
        </div>
      </div>
    </div>
    <!-- Overflow menu for users -->
    <Popover
      ref="moreUsersPopoverRef"
      popDown
      :anchorTo="moreUsersButtonRef"
      onTopOfEverything
    >
      <div
        class="flex flex-col rounded bg-shade-0 dark:bg-shade-100 border dark:border-neutral-500"
      >
        <div
          class="w-full text-center text-xs italic p-xs text-neutral-500 dark:text-neutral-400"
        >
          {{ sortedUsers.length }} Users Online
        </div>
        <SiSearch
          class="pt-0"
          placeholder="search users"
          @search="onSearchUpdated"
        />

        <div
          class="flex flex-col max-w-[250px] max-h-[60vh] overflow-x-hidden overflow-y-auto"
        >
          <UserCard
            v-for="(user, index) in filteredUsers"
            :key="index"
            :user="user"
            iconHasHoverState
          />
        </div>
      </div>
    </Popover>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { SiSearch, Icon } from "@si/vue-lib/design-system";
import clsx from "clsx";
import Popover from "@/components/Popover.vue";
import { usePresenceStore } from "@/store/presence.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import UserIcon from "./UserIcon.vue";
import UserCard from "./UserCard.vue";

const presenceStore = usePresenceStore();
const changeSetsStore = useChangeSetsStore();

export type UserInfo = {
  name: string;
  color?: string | null;
  status?: string | null;
  changeSet?: string; // TODO(Wendy) - this should be called changeSetId
  pictureUrl?: string | null;
  view?: string; // TODO(Wendy) - this should be called viewId
  // TODO(Wendy) - we should probably also send the viewName so we can show it in the tooltip
};

const moreUsersPopoverRef = ref();
const moreUsersButtonRef = ref();

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const testUsers = (amount = 10) => {
  return Array.from(Array(amount).keys()).map((i) => ({
    name: `test user ${i}`,
  }));
};

const users = computed<UserInfo[]>(() => {
  // Uncomment this line when you need to see users in the Collaborators widget for testing
  // return testUsers();

  const list = [] as UserInfo[];
  for (const user of _.values(presenceStore.usersById)) {
    list.push({
      name: user.name,
      color: user.color,
      status: user.idle ? "idle" : "active",
      changeSet: user.changeSetId,
      pictureUrl: user.pictureUrl,
      view: user.viewId,
    });
  }

  return list;
});

const sortedUsers = computed(() => {
  const usersCopy = _.clone(users.value);
  return usersCopy.sort((a, b) => {
    if (changeSetsStore.selectedChangeSetId) {
      if (
        a.changeSet !== changeSetsStore.selectedChangeSetId &&
        b.changeSet === changeSetsStore.selectedChangeSetId
      ) {
        return 2;
      }
      if (
        a.changeSet === changeSetsStore.selectedChangeSetId &&
        b.changeSet !== changeSetsStore.selectedChangeSetId
      ) {
        return -2;
      }
    }
    if (a.status === "idle" && b.status !== "idle") return 1;
    if (a.status !== "idle" && b.status === "idle") return -1;
    return 0;
  });
});

const displayUsers = computed(() => {
  if (sortedUsers.value.length < 7) return sortedUsers.value;
  else {
    const displayUsers = sortedUsers.value.slice(0, 5);

    return displayUsers;
  }
});

const moreUsersNumber = computed(() => {
  if (showOneIcon.value) return sortedUsers.value.length;
  else return sortedUsers.value.length - 5;
});

const userTooltips = computed(() => {
  const tooltips = [] as {
    content: string;
    theme: string;
  }[];

  displayUsers.value.forEach((user) => {
    tooltips.push({
      content: `<div class='flex flex-col items-center max-w-lg'>
        <div class='text-center font-bold w-full break-words line-clamp-3 pb-3xs px-sm min-w-0'>${
          user.name
        }</div>
        <div class='text-xs font-bold w-full text-center line-clamp-3 px-sm'>${
          user.changeSet
            ? changeSetsStore.changeSetsById[user.changeSet]?.name || "Head"
            : "Head"
        }</div>
        <div class='text-xs w-full text-center line-clamp-3 px-sm'>${
          user.status
        }</div>
        </div>`,
      theme: "user-info",
    });
  });

  return tooltips;
});

const moreUsersTooltip = computed(() => {
  let content;

  if (showOneIcon.value) {
    content = `<div class="px-xs font-bold">${moreUsersNumber.value} Editors Online</div>`;
  } else {
    content = `<div class="px-xs font-bold">${moreUsersNumber.value} More Online</div>`;
  }

  return {
    content,
    theme: "user-info",
  };
});

const width = computed(() => {
  if (showOneIcon.value || sortedUsers.value.length === 1) {
    return "w-8";
  } else if (sortedUsers.value.length < 3) {
    return "w-12";
  } else if (sortedUsers.value.length < 5) {
    return "w-20";
  } else {
    return "w-28";
  }
});

const openMoreUsersPopover = () => {
  moreUsersPopoverRef.value.open();
};

const windowWidth = ref(window.innerWidth);

const windowResizeHandler = () => {
  windowWidth.value = window.innerWidth;
};

onMounted(() => {
  windowResizeHandler();
  window.addEventListener("resize", windowResizeHandler);
});
onBeforeUnmount(() => {
  window.removeEventListener("resize", windowResizeHandler);
});

const showOneIcon = computed(() => windowWidth.value < 850);

const filterString = ref("");
const filterStringCleaned = computed(() => {
  if (!moreUsersPopoverRef.value.isOpen) return "";
  return filterString.value.trim().toLowerCase();
});
function onSearchUpdated(newFilterString: string) {
  filterString.value = newFilterString;
}
const filterModeActive = computed(() => !!filterStringCleaned.value);

const filteredUsers = computed(() => {
  if (filterModeActive.value) {
    return sortedUsers.value.filter((user) =>
      user.name.toLowerCase().includes(filterStringCleaned.value),
    );
  } else return sortedUsers.value;
});
</script>
