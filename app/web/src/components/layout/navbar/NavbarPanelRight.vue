<template>
  <div class="flex flex-row flex-1 basis-1/2 items-center min-w-0 h-full justify-end">
    <template v-if="!invalidWorkspace">
      <Collaborators />
      <Notifications />
    </template>

    <template v-if="!collapse">
      <NavbarButton tooltipText="Documentation" icon="question-circle" externalLinkTo="https://docs.systeminit.com/" />

      <NavbarButton
        tooltipText="Discord Community"
        icon="logo-discord"
        externalLinkTo="https://discord.gg/system-init"
      />

      <WorkspaceSettingsMenu />
    </template>

    <ProfileButton :showTopLevelMenuItems="collapse" />
  </div>
</template>

<script lang="ts" setup>
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import NavbarButton from "./NavbarButton.vue";
import Collaborators from "./Collaborators.vue";
import Notifications from "./Notifications.vue";
import WorkspaceSettingsMenu from "./WorkspaceSettingsMenu.vue";
import ProfileButton from "./ProfileButton.vue";

defineProps({
  invalidWorkspace: { type: Boolean },
});

const windowWidth = ref(window.innerWidth);
const collapse = computed(() => windowWidth.value < 1200);

const windowResizeHandler = () => {
  windowWidth.value = window.innerWidth;
};

onMounted(() => {
  windowResizeHandler();
  window.addEventListener("resize", windowResizeHandler);
});
onBeforeUnmount(() => {
  window.removeEventListener("resize", windowResizeHandler);
});
</script>
