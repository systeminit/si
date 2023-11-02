<template>
  <div
    :class="
      clsx(
        'flex flex-row justify-evenly items-center overflow-hidden m-xs',
        width,
      )
    "
  >
    <template v-if="!(showOneIcon && displayUsers.length > 1)">
      <div v-for="(user, index) in displayUsers" :key="index" class="h-8">
        <UserIcon
          :tooltip="userTooltips[index]"
          :user="user"
          class="absolute translate-x-[-50%]"
        />
      </div>
    </template>

    <div
      v-if="sortedUsers.length !== 1 && (showOneIcon || sortedUsers.length > 6)"
      class="h-8"
    >
      <div
        ref="moreUsersButtonRef"
        v-tooltip="moreUsersTooltip"
        :class="
          clsx(
            'absolute translate-x-[-50%] h-8 w-8 cursor-pointer bg-black',
            'border-2 border-shade-0 rounded-full overflow-hidden',
            'flex flex-row items-center',
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
          <template v-if="moreUsersNumber < 10000"
            >+{{ moreUsersNumber }}</template
          >
          <template v-else>+</template>
        </div>
      </div>
    </div>
    <Popover
      ref="moreUsersPopoverRef"
      popDown
      :anchorTo="moreUsersButtonRef"
      onTopOfEverything
    >
      <div
        class="flex flex-col rounded bg-shade-0 dark:bg-shade-100 border dark:border-neutral-500"
      >
        <SiSearch
          placeholder="search users"
          autoSearch
          @search="onSearchUpdated"
        />
        <div
          class="flex flex-col max-w-[250px] max-h-[60vh] overflow-x-hidden overflow-y-auto"
        >
          <div
            v-for="(user, index) in filteredUsers"
            :key="index"
            class="flex flex-row items-center gap-xs p-xs cursor-pointer overflow-hidden hover:bg-action-200 dark:hover:bg-action-500 flex-none"
          >
            <UserIcon :user="user" />

            <div class="flex flex-col min-w-0">
              <div class="w-full truncate leading-tight">
                {{ user.name }}
              </div>
              <div class="text-xs italic">
                {{ user.status }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </Popover>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import clsx from "clsx";
import Popover from "@/components/Popover.vue";
import SiSearch from "@/components/SiSearch.vue";
import UserIcon from "./UserIcon.vue";

export type UserInfo = {
  name: string;
  color?: string;
  status?: string;
  pictureUrl?: string;
};

const moreUsersPopoverRef = ref();
const moreUsersButtonRef = ref();

const users: UserInfo[] = [
  { name: "wandy", color: "#00f", status: "active" },
  { name: "wendi", color: "#f00", status: "idle" },
  { name: "wundee", color: "#ff0", status: "active" },
  { name: "window", color: "#0f0", status: "active" },
  { name: "wahndie", color: "#0ff", status: "idle" },
  { name: "whatever", color: "#f0f", status: "idle" },
  { name: "user7777777", color: "#f90", status: "idle" },
  { name: "user8", color: "#f90", status: "idle" },
  { name: "user9", color: "#f90", status: "active" },
  { name: "user10", color: "#f90", status: "idle" },
  { name: "user11", color: "#f90", status: "idle" },
  { name: "user12", color: "#f90", status: "idle" },
  { name: "user8", color: "#f90", status: "idle" },
  { name: "user9", color: "#f90", status: "active" },
  { name: "user10", color: "#f90", status: "idle" },
  { name: "user11", color: "#f90", status: "idle" },
  { name: "user12", color: "#f90", status: "idle" },
  { name: "user8", color: "#f90", status: "idle" },
  { name: "user9", color: "#f90", status: "active" },
  { name: "user10", color: "#f90", status: "idle" },
  { name: "user11", color: "#f90", status: "idle" },
  { name: "user12", color: "#f90", status: "idle" },
  { name: "user8", color: "#f90", status: "idle" },
  { name: "user9", color: "#f90", status: "active" },
  { name: "user10", color: "#f90", status: "idle" },
  { name: "user11", color: "#f90", status: "idle" },
  { name: "user12", color: "#f90", status: "idle" },
  {
    name: "thisuserhasareallylongnameomgsolongthisuserhasareallylongnameomgsolongthisuserhasareallylongnameomgsolongthisuserhasareallylongnameomgsolong",
    color: "#f90",
    status: "idle",
  },
];

const sortedUsers = computed(() => {
  const usersCopy = _.clone(users);
  return usersCopy.sort((a, b) => {
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
      content: `<div class='flex flex-col items-center'><div class='font-bold'>${user.name}</div><div class='text-xs'>${user.status}</div></div>`,
      theme: "user-info",
    });
  });

  return tooltips;
});

const moreUsersTooltip = computed(() => {
  let content;

  if (showOneIcon.value) {
    content = `${moreUsersNumber.value} Editors Online`;
  } else {
    content = `${moreUsersNumber.value} More Online`;
  }

  return {
    content,
    theme: "user-info",
  };
});

const width = computed(() => {
  windowResizeHandler();
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

const showOneIcon = ref(false);

const windowResizeHandler = () => {
  if (window.innerWidth < 900) {
    showOneIcon.value = true;
  } else {
    showOneIcon.value = false;
  }
};

onMounted(() => {
  window.addEventListener("resize", windowResizeHandler);
});
onBeforeUnmount(() => {
  window.removeEventListener("resize", windowResizeHandler);
});

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
