<template>
  <Disclosure v-slot="{ open }" as="nav" :class="bgColor">
    <div class="pl-2 dark:border-b-2 dark:border-[#525252]">
      <div class="flex items-center h-16">
        <!-- Left side -->
        <div class="flex items-center justify-center place-items-center">
          <img
            class="block h-11 w-11 my-2 mr-2 bg-black"
            :src="SiLogoWts"
            alt="SI Logo"
          />

          <div v-if="workspace && workspaceList" class="flex-col my-2 ml-2 p-1">
            <div class="text-white text-xs font-medium">WORKSPACE:</div>
            <SiSelect
              id="workspace-selector"
              v-model="selectedWorkspaceName"
              tooltip-text="Workspace selector"
              name="workspaceSelect"
              class=""
              :options="workspaceList"
              :navbar-mode="true"
            />
          </div>
        </div>

        <!-- Center -->
        <ViewerPanel />

        <!-- Right -->
        <ButtonPanel />

        <!-- Mobile menu button -->
        <div class="-mr-2 flex sm:hidden">
          <DisclosureButton
            class="inline-flex items-center justify-center p-2 rounded-md text-gray-400 hover:text-white hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
          >
            <span class="sr-only">Open main menu</span>
            <MenuIcon v-if="!open" class="block h-6 w-6" aria-hidden="true" />
            <XIcon v-else class="block h-6 w-6" aria-hidden="true" />
          </DisclosureButton>
        </div>
      </div>
    </div>
  </Disclosure>
</template>

<script setup lang="ts">
import { Disclosure, DisclosureButton } from "@headlessui/vue";
import { MenuIcon, XIcon } from "@heroicons/vue/outline";
import SiLogoWts from "@/assets/images/si-logo-wts.svg";
import { refFrom } from "vuse-rx";
import { WorkspaceService } from "@/service/workspace";
import { Workspace } from "@/api/sdf/dal/workspace";
import SiSelect from "@/atoms/SiSelect.vue";
import { LabelList } from "@/api/sdf/dal/label_list";
import { computed } from "vue";
import ButtonPanel from "@/molecules/SiNavbarPanels/ButtonPanel.vue";
import ViewerPanel from "@/molecules/SiNavbarPanels/ViewerPanel.vue";

const workspace = refFrom<Workspace | null>(
  WorkspaceService.currentWorkspace(),
);

// FIXME(nick): selecting the workspace should change to that workspace.
// Perhaps, this should work similarly to "switchTo" from "SystemService".
const selectedWorkspaceName = computed(() => {
  if (workspace.value) {
    return workspace.value.name;
  }
  return "- none -";
});

// FIXME(nick): this should contain a real list of available workspaces.
const workspaceList = computed((): LabelList<string> | false => {
  if (workspace.value) {
    let labels: LabelList<string> = [];
    labels.push({
      label: workspace.value.name,
      value: workspace.value.name,
    });
    return labels;
  }
  return false;
});

const bgColor = "bg-[#333333]";
</script>
