<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <NavbarButton tooltipText="Workspace Settings">
    <Icon name="settings" />

    <template #dropdownContent>
      <DropdownMenuItem
        v-if="featureFlagsStore.INVITE_USER"
        icon="user-circle"
        label="Invite User"
        @click="inviteUserModalRef?.open()"
      />
      <DropdownMenuItem
        icon="cloud-upload"
        label="Export Workspace"
        @click="exportModalRef?.open()"
      />
      <DropdownMenuItem
        icon="cloud-download"
        label="Import Workspace"
        @click="importModalRef?.open()"
      />
    </template>
  </NavbarButton>

  <WorkspaceImportModal ref="importModalRef" />
  <WorkspaceExportModal ref="exportModalRef" />
  <WorkspaceInviteUserModal ref="inviteUserModalRef" />
</template>

<script setup lang="ts">
import { DropdownMenuItem, Icon } from "@si/vue-lib/design-system";
import { ref } from "vue";
import WorkspaceImportModal from "@/components/WorkspaceImportModal.vue";
import WorkspaceExportModal from "@/components/WorkspaceExportModal.vue";
import WorkspaceInviteUserModal from "@/components/WorkspaceInviteUserModal.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import NavbarButton from "./NavbarButton.vue";

const featureFlagsStore = useFeatureFlagsStore();

const importModalRef = ref<InstanceType<typeof WorkspaceImportModal>>();
const exportModalRef = ref<InstanceType<typeof WorkspaceExportModal>>();
const inviteUserModalRef = ref<InstanceType<typeof WorkspaceInviteUserModal>>();
</script>
