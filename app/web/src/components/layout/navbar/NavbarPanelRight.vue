<template>
  <div class="flex justify-end flex-1 min-w-0">
    <Collaborators v-if="featureFlagsStore.COLLABORATORS" />

    <NavbarButton tooltipText="Copy link" @click="copyURL">
      <Icon name="link" />
    </NavbarButton>

    <SiThemeSwitcher />

    <WorkspaceSettingsMenu v-if="featureFlagsStore.WORKSPACE_BACKUPS" />

    <ProfileButton />
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import SiThemeSwitcher from "./NavbarThemeSwitcher.vue";
import NavbarButton from "./NavbarButton.vue";
import Collaborators from "./Collaborators.vue";
import WorkspaceSettingsMenu from "./WorkspaceSettingsMenu.vue";
import ProfileButton from "./ProfileButton.vue";

const featureFlagsStore = useFeatureFlagsStore();

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
};
</script>
