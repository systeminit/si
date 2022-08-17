<template>
  <nav class="bg-neutral-900 text-white relative">
    <div class="pl-2 border-b-2 border-neutral-300 dark:border-neutral-600">
      <div class="flex items-center h-16">
        <!-- Left side -->
        <div class="flex items-center justify-center place-items-center h-full">
          <img
            class="block h-11 w-11 my-2 mr-2 bg-black"
            :src="SiLogoWts"
            alt="SI Logo"
          />

          <SiBarButton tooltip-text="Workspaces">
            <template #default="{ hovered, open }">
              <div class="flex-col flex text-left">
                <div class="text-xs font-medium">WORKSPACE:</div>
                <div class="flex-row flex font-semibold">
                  <span>{{ selectedWorkspaceName }}</span>
                  <SiArrow :nudge="hovered || open" class="ml-1 w-5" />
                </div>
              </div>
            </template>

            <template #dropdownContent>
              <SiDropdownItem checked class="text-sm">{{
                selectedWorkspaceName
              }}</SiDropdownItem>
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
import SiLogoWts from "@/assets/images/si-logo-wts.svg";
import { refFrom } from "vuse-rx";
import { WorkspaceService } from "@/service/workspace";
import { Workspace } from "@/api/sdf/dal/workspace";
import { computed } from "vue";
import NavbarPanelRight from "@/organisms/NavbarPanelRight.vue";
import NavbarPanelCenter from "@/organisms/NavbarPanelCenter.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiBarButton from "@/molecules/SiBarButton.vue";
import SiArrow from "@/atoms/SiArrow.vue";

const workspace = refFrom<Workspace | null>(
  WorkspaceService.currentWorkspace(),
);

// FIXME(nick): this should contain a real list of available workspaces. This
// will likely be an observable.
const selectedWorkspaceName = computed(() => {
  if (workspace.value) {
    return workspace.value.name;
  }
  return "- none -";
});
</script>
