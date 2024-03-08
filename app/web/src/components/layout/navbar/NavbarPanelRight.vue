<template>
  <div class="flex items-center justify-end h-full flex-1 min-w-0">
    <Collaborators />

    <NavbarButton tooltipText="Copy link" @click="copyURL">
      <Icon name="link" />
    </NavbarButton>

    <SiThemeSwitcher />

    <WorkspaceSettingsMenu />

    <ProfileButton />
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import SiThemeSwitcher from "./NavbarThemeSwitcher.vue";
import NavbarButton from "./NavbarButton.vue";
import Collaborators from "./Collaborators.vue";
import WorkspaceSettingsMenu from "./WorkspaceSettingsMenu.vue";
import ProfileButton from "./ProfileButton.vue";

const copyURL = () => {
  navigator.clipboard.writeText(window.location.href);
};
</script>
