<template>
  <div class="flex items-center h-full">
    <NavbarButton tooltipText="Copy link" @click="copyURL">
      <Icon name="link" />
    </NavbarButton>

    <SiThemeSwitcher />

    <WorkspaceSettingsMenu v-if="featureFlagsStore.WORKSPACE_BACKUPS" />

    <NavbarButton tooltipText="Profile">
      <template #default="{ open, hovered }">
        <div class="flex-row flex text-white items-center">
          <img
            v-if="authStore.user?.picture_url"
            class="h-8 w-8 rounded-full bg-white border-black border-2"
            :src="authStore.user?.picture_url"
            referrerpolicy="no-referrer"
          />
          <Icon v-else name="user-circle" />
          <SiArrow :nudge="open || hovered" class="ml-1" />
        </div>
      </template>

      <template #dropdownContent>
        <DropdownMenuItem
          linkToNamedRoute="logout"
          icon="logout"
          label="Logout"
        />
        <DropdownMenuItem
          v-if="isDevMode"
          linkToNamedRoute="workspace-dev-dashboard"
          icon="cat"
          label="Dev Dashboard"
        />
      </template>
    </NavbarButton>
  </div>
</template>

<script lang="ts" setup>
import { Icon, DropdownMenuItem } from "@si/vue-lib/design-system";
import SiArrow from "@/components/SiArrow.vue";
import { useAuthStore } from "@/store/auth.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import SiThemeSwitcher from "./NavbarThemeSwitcher.vue";
import NavbarButton from "./NavbarButton.vue";
import WorkspaceSettingsMenu from "./WorkspaceSettingsMenu.vue";

const featureFlagsStore = useFeatureFlagsStore();

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
};

// Cannot use inside the template directly.
const isDevMode = import.meta.env.DEV;

const authStore = useAuthStore();
</script>
