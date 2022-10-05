<template>
  <nav class="bg-neutral-900 text-white relative">
    <div class="pl-2 border-b-2 border-neutral-300 dark:border-neutral-600">
      <div class="flex items-center h-16">
        <!-- Left side -->
        <div class="flex items-center justify-center place-items-center h-full">
          <img class="block h-11 w-11 my-2 mr-2" :src="logo" alt="SI Logo" />

          <SiBarButton tooltip-text="Workspaces">
            <template #default="{ hovered, open }">
              <div class="flex-col flex text-left">
                <div class="text-xs font-medium">WORKSPACE:</div>
                <div class="flex-row flex font-semibold">
                  <span v-if="workspacesReqStatus.isPending">loading...</span>
                  <template v-else>
                    <span>{{ selectedWorkspace?.name || "- none -" }}</span>
                    <SiArrow :nudge="hovered || open" class="ml-1 w-5" />
                  </template>
                </div>
              </div>
            </template>

            <template #dropdownContent>
              <SiDropdownItem
                v-for="workspace in workspaces"
                :key="workspace.id"
                :checked="workspace.id === selectedWorkspaceId"
                class="text-sm"
              >
                {{ workspace.name }}
              </SiDropdownItem>
            </template>
          </SiBarButton>
        </div>

        <!-- Center -->
        <NavbarPanelCenter />

        <!-- Right -->
        <NavbarPanelRight />
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { computed } from "vue";
import SiLogoWts from "@/assets/images/si-logo-wts.svg?url";
import SiLogoWtsDev from "@/assets/images/si-logo-wts-dev.svg?url";
import NavbarPanelRight from "@/organisms/NavbarPanelRight.vue";
import NavbarPanelCenter from "@/organisms/NavbarPanelCenter.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiBarButton from "@/molecules/SiBarButton.vue";
import SiArrow from "@/atoms/SiArrow.vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "@/store/change_sets.store";

const isDevMode = import.meta.env.DEV;

const logo = computed(() => (isDevMode ? SiLogoWtsDev : SiLogoWts));

const workspacesStore = useWorkspacesStore();
const workspaces = computed(() => workspacesStore.allWorkspaces);
const selectedWorkspaceId = computed(() => workspacesStore.selectedWorkspaceId);
const selectedWorkspace = computed(() => workspacesStore.selectedWorkspace);

const workspacesReqStatus = workspacesStore.getRequestStatus(
  "FETCH_USER_WORKSPACES",
);

// including here so the changeset store always has something using it
// TODO: may want to move this to main app or app layout...
// but basically it should always be loaded when we are logged in (on app pages)
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const changeSetsStore = useChangeSetsStore();
</script>
