<template>
  <nav
    class="bg-neutral-900 text-white relative border-b border-shade-100 shadow-[0_4px_4px_0_rgba(0,0,0,0.15)] z-90"
  >
    <div class="pl-sm">
      <div class="flex items-center h-16">
        <!-- Left side -->
        <div class="flex items-center justify-center place-items-center h-full">
          <SiLogo class="block h-10 w-10 my-2 mr-2" />

          <div class="flex flex-col gap-1 ml-4">
            <div class="text-xs font-medium capsize">WORKSPACE:</div>

            <VormInput
              class="flex-grow font-bold"
              size="sm"
              type="dropdown"
              noLabel
              :modelValue="workspacesStore.selectedWorkspacePk"
              :options="workspaceDropdownOptions"
            />
          </div>

          <Icon name="slash" size="2xl" tone="neutral" />

          <ChangeSetPanel />
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
import { Icon, VormInput } from "@si/vue-lib/design-system";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import * as _ from "lodash-es";
import { useWorkspacesStore } from "@/store/workspaces.store";
import ChangeSetPanel from "@/components/layout/navbar/ChangeSetPanel.vue";
import NavbarPanelCenter from "./NavbarPanelCenter.vue";
import NavbarPanelRight from "./NavbarPanelRight.vue";

const workspacesStore = useWorkspacesStore();

const workspaceDropdownOptions = computed(() =>
  _.map(workspacesStore.allWorkspaces ?? [], (w) => ({
    value: w.pk,
    label: w.name,
  })),
);
</script>
