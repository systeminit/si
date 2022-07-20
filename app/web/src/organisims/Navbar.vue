<template>
  <nav class="bg-[#333333] text-white relative">
    <div class="pl-2 dark:border-b-2 dark:border-[#525252]">
      <div class="flex items-center h-16">
        <!-- Left side -->
        <div class="flex items-center justify-center place-items-center h-full">
          <img
            class="block h-11 w-11 my-2 mr-2 bg-black"
            :src="SiLogoWts"
            alt="SI Logo"
          />

          <SiNavbarButton
            v-slot="{ hovered, open }"
            tooltip-text="Workspaces"
            :options="workspaceList"
            :text-mode="true"
            dropdown-classes="text-center"
          >
            <div class="flex-col flex text-left">
              <div class="text-xs font-medium">WORKSPACE:</div>
              <div class="flex-row flex font-semibold">
                <span>
                  {{ workspace?.name || "- none -" }}
                </span>
                <SiArrow :nudge="hovered || open" class="ml-1 w-5" />
              </div>
            </div>
          </SiNavbarButton>
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
import NavbarPanelRight from "@/organisims/NavbarPanelRight.vue";
import NavbarPanelCenter from "@/organisims/NavbarPanelCenter.vue";
import { SiDropdownOption } from "@/atoms/SiDropdown.vue";
import SiNavbarButton from "@/molecules/SiNavbarButton.vue";
import SiArrow from "@/atoms/SiArrow.vue";

const workspace = refFrom<Workspace | null>(
  WorkspaceService.currentWorkspace(),
);

// FIXME(nick): this should contain a real list of available workspaces. This
// will likely be an observable.
const workspaceList = computed((): SiDropdownOption[] => {
  let options: SiDropdownOption[] = [];
  if (workspace.value) {
    options.push({
      text: workspace.value.name,
      action: () => {
        console.log("selected workspace! huzzah.");
      },
    });
  }
  return options;
});
</script>
