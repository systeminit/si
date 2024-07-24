<template>
  <div class="flex items-center justify-end h-full flex-1 min-w-0">
    <Collaborators />

    <NavbarButton
      tooltipText="Documentation"
      externalLinkTo="https://docs.systeminit.com/"
    >
      <Icon name="question-circle" />
    </NavbarButton>

    <NavbarButton
      tooltipText="Discord Community"
      externalLinkTo="https://discord.gg/system-init"
    >
      <Icon name="logo-discord" />
    </NavbarButton>

    <WorkspaceSettingsMenu />

    <ProfileButton />
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import NavbarButton from "./NavbarButton.vue";
import Collaborators from "./Collaborators.vue";
import WorkspaceSettingsMenu from "./WorkspaceSettingsMenu.vue";
import ProfileButton from "./ProfileButton.vue";
</script>
