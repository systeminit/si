<template>
  <div class="flex items-center justify-end h-full flex-1 min-w-0">
    <Collaborators />

    <NavbarButton
      tooltipText="Documentation"
      icon="question-circle"
      externalLinkTo="https://docs.systeminit.com/"
    />

    <NavbarButton
      tooltipText="Discord Community"
      icon="logo-discord"
      externalLinkTo="https://discord.gg/system-init"
    />

    <WorkspaceSettingsMenu />

    <ProfileButton />
  </div>
</template>

<script lang="ts" setup>
import NavbarButton from "./NavbarButton.vue";
import Collaborators from "./Collaborators.vue";
import WorkspaceSettingsMenu from "./WorkspaceSettingsMenu.vue";
import ProfileButton from "./ProfileButton.vue";
</script>
