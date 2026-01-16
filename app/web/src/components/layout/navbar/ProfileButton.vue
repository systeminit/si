<template>
  <NavbarButton ref="navbarButtonRef" class="flex-none py-xs" tooltipText="Profile">
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

      <template v-if="showTopLevelMenuItems">
        <WorkspaceImportModal ref="importModalRef" />
        <WorkspaceExportModal ref="exportModalRef" />
        <WorkspaceIntegrationsModal ref="integrationsModalRef" />
      </template>
    </template>

    <template #dropdownContent>
      <DropdownMenuItem icon="user-circle" label="Profile" @click="openWorkspaceDetailsHandler" />
      <DropdownMenuItem :icon="themeIcon" label="Change Theme" @click="changeTheme" />
      <DropdownMenuItem v-if="isDevMode" icon="cat" label="Dev Dashboard" linkToNamedRoute="workspace-dev-dashboard" />
      <DropdownMenuItem
        v-if="featureFlagsStore.ADMIN_PANEL_ACCESS"
        icon="alert-triangle"
        label="Admin Dashboard"
        linkToNamedRoute="workspace-admin-dashboard"
      />
      <template v-if="showTopLevelMenuItems">
        <DropdownMenuItem icon="question-circle" label="Documentation" href="https://docs.systeminit.com/" />
        <DropdownMenuItem icon="logo-discord" label="Discord Community" href="https://discord.gg/system-init" />
        <DropdownMenuItem icon="settings" label="Workspace Settings" @click="openSettings" />
      </template>
      <DropdownMenuItem class="profile-dropdown-menu-logout" icon="logout" label="Logout" linkToNamedRoute="logout" />
    </template>
    <template #dropdownContentSecondary>
      <WorkspaceSettingsMenuItems
        v-if="secondaryMenu === 'settings'"
        @openImportModal="importModalRef?.open()"
        @openExportModal="exportModalRef?.open()"
        @openIntegrationsModal="integrationsModalRef?.open()"
      />
      <template v-else>
        <DropdownMenuItem checkable :checked="!userOverrideTheme" icon="bolt" @select="userOverrideTheme = null">
          System theme
        </DropdownMenuItem>
        <DropdownMenuItem
          checkable
          :checked="userOverrideTheme === 'light'"
          icon="sun"
          @select="userOverrideTheme = 'light'"
        >
          Light theme
        </DropdownMenuItem>
        <DropdownMenuItem
          checkable
          :checked="userOverrideTheme === 'dark'"
          icon="moon"
          @select="userOverrideTheme = 'dark'"
        >
          Dark theme
        </DropdownMenuItem>
      </template>
    </template>
  </NavbarButton>
</template>

<script lang="ts" setup>
import { DropdownMenuItem, userOverrideTheme } from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import SiArrow from "@/components/SiArrow.vue";
import { useAuthStore } from "@/store/auth.store";
import { isDevMode } from "@/utils/debug";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import WorkspaceImportModal from "@/components/WorkspaceImportModal.vue";
import WorkspaceExportModal from "@/components/WorkspaceExportModal.vue";
import WorkspaceIntegrationsModal from "@/components/WorkspaceIntegrationsModal.vue";
import NavbarButton from "./NavbarButton.vue";
import UserIcon from "./UserIcon.vue";
import WorkspaceSettingsMenuItems from "./WorkspaceSettingsMenuItems.vue";

const importModalRef = ref<InstanceType<typeof WorkspaceImportModal>>();
const exportModalRef = ref<InstanceType<typeof WorkspaceExportModal>>();
const integrationsModalRef = ref<InstanceType<typeof WorkspaceIntegrationsModal>>();

defineProps({
  showTopLevelMenuItems: { type: Boolean },
});

const featureFlagsStore = useFeatureFlagsStore();

const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;
const authStore = useAuthStore();
const navbarButtonRef = ref<InstanceType<typeof NavbarButton>>();

const openWorkspaceDetailsHandler = () => {
  window.open(`${AUTH_PORTAL_URL}/profile`, "_blank");
};

const themeIcon = computed(() => {
  if (userOverrideTheme.value === "light") return "sun";
  else if (userOverrideTheme.value === "dark") return "moon";
  else return "bolt";
});

const secondaryMenu = ref<"theme" | "settings">("theme");

const changeTheme = () => {
  secondaryMenu.value = "theme";
  navbarButtonRef.value?.openSecondary();
};

const openSettings = () => {
  secondaryMenu.value = "settings";
  navbarButtonRef.value?.openSecondary();
};
</script>
