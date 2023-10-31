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
      <div v-for="(user, index) in displayUsers" :key="user.name" class="h-8">
        <div
          v-tooltip="userTooltips[index]"
          class="absolute translate-x-[-50%] h-8 w-8 border-2 rounded-full cursor-pointer"
          :style="`border-color: ${user.color}`"
        >
          <!-- TODO(Wendy) - This should check for and pull the image of the user in question, not the current user's image! -->
          <img
            v-if="true"
            class="rounded-full bg-white"
            :src="authStore.user?.picture_url"
            referrerpolicy="no-referrer"
          />
          <Icon v-else name="user-circle" size="full" />
        </div>
      </div>
    </template>
    <div
      v-if="users.length !== 1 && (showOneIcon || users.length > 6)"
      class="h-8"
    >
      <div
        v-tooltip="moreUsersTooltip"
        class="absolute translate-x-[-50%] h-8 w-8 border-2 border-shade-0 rounded-full cursor-pointer bg-black"
      >
        <Icon name="dots-horizontal" size="full" />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useAuthStore } from "@/store/auth.store";
import { Icon } from "@si/vue-lib/design-system";
import clsx from "clsx";

const authStore = useAuthStore();

const users = [
  { name: "wendi", color: "#f00" },
  { name: "wandy", color: "#00f" },
  { name: "wundee", color: "#ff0" },
  { name: "window", color: "#0f0" },
  { name: "wahndie", color: "#0ff" },
  { name: "whatever", color: "#f0f" },
  { name: "user7", color: "#f90" },
  { name: "user8", color: "#f90" },
  { name: "user9", color: "#f90" },
];

const displayUsers = computed(() => {
  if (users.length < 7) return users;
  else {
    const displayUsers = users.slice(0, 5);

    return displayUsers;
  }
});

const userTooltips = computed(() => {
  const tooltips = [] as {
    content: string;
    theme: string;
  }[];

  displayUsers.value.forEach((user) => {
    tooltips.push({
      content: user.name,
      theme: "user-info",
    });
  });

  return tooltips;
});

const moreUsersTooltip = computed(() => {
  let content;

  if (showOneIcon.value) {
    content = `${users.length} Editors Online`;
  } else {
    content = `${users.length - 5} More Online`;
  }

  return { content, theme: "user-info" };
});

const width = computed(() => {
  windowResizeHandler();
  if (showOneIcon.value || users.length === 1) {
    return "w-8";
  } else if (users.length < 3) {
    return "w-12";
  } else if (users.length < 5) {
    return "w-20";
  } else {
    return "w-28";
  }
});

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
</script>
