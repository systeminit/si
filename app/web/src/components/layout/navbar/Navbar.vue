<template>
  <nav
    class="bg-neutral-900 text-white relative border-b border-shade-100 shadow-[0_4px_4px_0_rgba(0,0,0,0.15)] z-100"
  >
    <div class="pl-2">
      <div class="flex items-center h-16">
        <!-- Left side -->
        <div class="flex items-center justify-center place-items-center h-full">
          <img
            class="block h-11 w-11 my-2 mr-2"
            :src="SiLogoUrl"
            alt="SI Logo"
          />

          <NavbarButton tooltip-text="Workspaces">
            <template #default="{ open, hovered }">
              <div class="flex-col flex text-left gap-xs">
                <div class="text-xs font-medium capsize pt-2xs">WORKSPACE:</div>
                <div class="flex-row flex font-semibold">
                  <span v-if="workspacesReqStatus.isPending">loading...</span>
                  <template v-else>
                    <span>{{ selectedWorkspace?.name || "- none -" }}</span>
                    <SiArrow :nudge="open || hovered" class="ml-1" />
                  </template>
                </div>
              </div>
            </template>

            <template #dropdownContent>
              <DropdownMenuItem
                v-for="workspace in workspaces"
                :key="workspace.pk"
                :checked="workspace.pk === selectedWorkspacePk"
                :label="workspace.name"
              />
            </template>
          </NavbarButton>
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
import { DropdownMenuItem } from "@si/vue-lib/design-system";
import SiLogoUrl from "@si/vue-lib/brand-assets/si-logo.svg?url";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import SiArrow from "@/components/SiArrow.vue";
import NavbarPanelCenter from "./NavbarPanelCenter.vue";
import NavbarPanelRight from "./NavbarPanelRight.vue";
import NavbarButton from "./NavbarButton.vue";

const workspacesStore = useWorkspacesStore();
const workspaces = computed(() => workspacesStore.allWorkspaces);
const selectedWorkspacePk = computed(() => workspacesStore.selectedWorkspacePk);
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
