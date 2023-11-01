<template>
  <div class="flex items-center justify-end h-full flex-1 min-w-0">
    <Collaborators v-if="featureFlagsStore.COLLABORATORS" />

    <NavbarButton tooltipText="Copy link" @click="copyURL">
      <Icon name="link" />
    </NavbarButton>

    <SiThemeSwitcher />

    <WorkspaceSettingsMenu v-if="featureFlagsStore.WORKSPACE_BACKUPS" />

    <NavbarButton tooltipText="Profile" class="flex-none">
      <template #default="{ open, hovered }">
        <div class="flex-row flex text-white items-center">
          <div class="h-8 w-8 border-2 border-black rounded-full">
            <img
              v-if="authStore.user?.picture_url"
              class="rounded-full bg-white"
              :src="authStore.user?.picture_url"
              referrerpolicy="no-referrer"
            />
            <Icon v-else name="user-circle" size="full" />
          </div>
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
import { useAuthStore } from "@/store/auth.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import SiArrow from "@/components/SiArrow.vue";
import SiThemeSwitcher from "./NavbarThemeSwitcher.vue";
import NavbarButton from "./NavbarButton.vue";
import Collaborators from "./Collaborators.vue";
import WorkspaceSettingsMenu from "./WorkspaceSettingsMenu.vue";

const featureFlagsStore = useFeatureFlagsStore();

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
};

// Cannot use inside the template directly.
const isDevMode = import.meta.env.DEV;

const authStore = useAuthStore();
</script>
