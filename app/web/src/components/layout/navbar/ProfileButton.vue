<template>
  <NavbarButton
    ref="navbarButtonRef"
    class="flex-none py-xs"
    tooltipText="Profile"
  >
    <template #default="{ open, hovered }">
      <div class="flex-row flex text-white items-center">
        <UserIcon
          :user="{
            name: authStore.user?.name ?? '',
            pictureUrl: authStore.user?.picture_url ?? null,
          }"
        />
        <SiArrow :nudge="open || hovered" class="ml-1" />
      </div>
    </template>

    <template #dropdownContent>
      <DropdownMenuItem
        icon="user-circle"
        label="Profile"
        @click="openWorkspaceDetailsHandler"
      />
      <DropdownMenuItem
        :icon="themeIcon"
        label="Change Theme"
        @click="changeTheme"
      />
      <DropdownMenuItem
        v-if="isDevMode"
        icon="cat"
        label="Dev Dashboard"
        linkToNamedRoute="workspace-dev-dashboard"
      />
      <DropdownMenuItem
        v-if="featureFlagsStore.ADMIN_PANEL_ACCESS"
        icon="alert-triangle"
        label="Admin Dashboard"
        linkToNamedRoute="workspace-admin-dashboard"
      />
      <DropdownMenuItem
        class="profile-dropdown-menu-logout"
        icon="logout"
        label="Logout"
        linkToNamedRoute="logout"
      />
    </template>
    <template #dropdownContentSecondary>
      <DropdownMenuItem
        :checked="!userOverrideTheme"
        icon="bolt"
        @select="userOverrideTheme = null"
      >
        System theme
      </DropdownMenuItem>
      <DropdownMenuItem
        :checked="userOverrideTheme === 'light'"
        icon="sun"
        @select="userOverrideTheme = 'light'"
      >
        Light theme
      </DropdownMenuItem>
      <DropdownMenuItem
        :checked="userOverrideTheme === 'dark'"
        icon="moon"
        @select="userOverrideTheme = 'dark'"
      >
        Dark theme
      </DropdownMenuItem>
    </template>
  </NavbarButton>
</template>

<script lang="ts" setup>
import { DropdownMenuItem, userOverrideTheme } from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import SiArrow from "@/components/SiArrow.vue";
import { useAuthStore } from "@/store/auth.store";
import { isDevMode } from "@/utils/debug";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import NavbarButton from "./NavbarButton.vue";
import UserIcon from "./UserIcon.vue";

const featureFlagsStore = useFeatureFlagsStore();

const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;
const workspacesStore = useWorkspacesStore();
const authStore = useAuthStore();
const navbarButtonRef = ref<InstanceType<typeof NavbarButton>>();

const openWorkspaceDetailsHandler = () => {
  const currentWorkspace = workspacesStore.urlSelectedWorkspaceId;
  if (!currentWorkspace) return;
  window.open(`${AUTH_PORTAL_URL}/profile`, "_blank");
};

const themeIcon = computed(() => {
  if (userOverrideTheme.value === "light") return "sun";
  else if (userOverrideTheme.value === "dark") return "moon";
  else return "bolt";
});

const changeTheme = () => {
  navbarButtonRef.value?.openSecondary();
};
</script>
