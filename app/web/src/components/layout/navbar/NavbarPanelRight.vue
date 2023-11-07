<template>
  <div class="flex items-center justify-end h-full flex-1 min-w-0">
    <Collaborators v-if="featureFlagsStore.COLLABORATORS" />

    <NavbarButton tooltipText="Copy link" @click="copyURL">
      <Icon name="link" />
    </NavbarButton>

    <SiThemeSwitcher />

    <WorkspaceSettingsMenu v-if="featureFlagsStore.WORKSPACE_BACKUPS" />

    <NavbarButton tooltipText="Profile" class="flex-none py-xs">
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
import UserIcon from "./UserIcon.vue";

const featureFlagsStore = useFeatureFlagsStore();

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
};

// Cannot use inside the template directly.
const isDevMode = import.meta.env.DEV;

const authStore = useAuthStore();
</script>
