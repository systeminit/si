<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <NavbarButton icon="settings" tooltipText="Workspace Settings">
    <template #dropdownContent>
      <DropdownMenuItem
        icon="settings"
        label="Manage Workspaces"
        @click="openManageWorkspacesHandler"
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
      <DropdownMenuItem
        icon="edit"
        label="Manage Users"
        @click="openWorkspaceDetailsHandler"
      />
      <DropdownMenuItem
        v-if="showViz"
        icon="diagram"
        label="Visualizer"
        @click="gotoViz"
      />
      <DropdownMenuItem
        icon="clipboard-copy"
        label="Copy Workspace Token"
        @click="copyWorkspaceToken"
      />
      <DropdownMenuItem
        v-if="featureFlagsStore.SLACK_WEBHOOK"
        icon="settings"
        label="Workspace Integrations"
        @click="integrationsModalRef?.open()"
      />
    </template>
  </NavbarButton>

  <WorkspaceImportModal ref="importModalRef" />
  <WorkspaceExportModal ref="exportModalRef" />
  <WorkspaceIntegrationsModal ref="integrationsModalRef" />
</template>

<script setup lang="ts">
import { DropdownMenuItem } from "@si/vue-lib/design-system";
import { ref, computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import WorkspaceImportModal from "@/components/WorkspaceImportModal.vue";
import WorkspaceExportModal from "@/components/WorkspaceExportModal.vue";
import WorkspaceIntegrationsModal from "@/components/WorkspaceIntegrationsModal.vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import NavbarButton from "./NavbarButton.vue";

const AUTH_PORTAL_URL = import.meta.env.VITE_AUTH_PORTAL_URL;
const importModalRef = ref<InstanceType<typeof WorkspaceImportModal>>();
const exportModalRef = ref<InstanceType<typeof WorkspaceExportModal>>();
const integrationsModalRef =
  ref<InstanceType<typeof WorkspaceIntegrationsModal>>();

const workspacesStore = useWorkspacesStore();
const changeSetStore = useChangeSetsStore();
const featureFlagsStore = useFeatureFlagsStore();
const router = useRouter();
const route = useRoute();

const openWorkspaceDetailsHandler = () => {
  const currentWorkspace = workspacesStore.urlSelectedWorkspaceId;
  if (!currentWorkspace) return;
  window.open(`${AUTH_PORTAL_URL}/workspace/${currentWorkspace}`, "_blank");
};

const openManageWorkspacesHandler = () => {
  const currentWorkspace = workspacesStore.urlSelectedWorkspaceId;
  if (!currentWorkspace) return;
  window.open(`${AUTH_PORTAL_URL}/workspaces/`, "_blank");
};

const showViz = computed(() => {
  return changeSetStore.selectedChangeSetId;
});

const gotoViz = () => {
  router.push({
    name: "workspace-viz",
    params: {
      ...route.params,
      changeSetId: changeSetStore.selectedChangeSetId,
    },
  });
};

const copyWorkspaceToken = () => {
  const currentWorkspace = workspacesStore.selectedWorkspace;
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  navigator.clipboard.writeText(currentWorkspace?.token || "");
};
</script>
