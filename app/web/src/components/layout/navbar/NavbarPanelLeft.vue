<template>
  <div class="flex items-center h-full flex-1">
    <SiLogo class="block h-10 w-10 my-2 mr-2 flex-none" />

    <div class="flex flex-col gap-1 ml-4 max-w-[50%]">
      <div class="text-xs font-medium capsize">WORKSPACE:</div>

      <VormInput
        v-model="selectedWorkspacePk"
        class="flex-grow font-bold"
        size="sm"
        type="dropdown"
        noLabel
        :options="workspaceDropdownOptions"
        placeholder="-- select a workspace --"
        @change="updateRoute"
      />
    </div>

    <Icon name="slash" size="2xl" tone="neutral" />

    <ChangeSetPanel class="max-w-[50%]" />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import { Icon, VormInput } from "@si/vue-lib/design-system";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { computed, ref, watch } from "vue";
import ChangeSetPanel from "./ChangeSetPanel.vue";

const workspacesStore = useWorkspacesStore();

const selectedWorkspacePk = ref<string | null>(null);
watch(
  () => workspacesStore.selectedWorkspacePk,
  () => {
    selectedWorkspacePk.value = workspacesStore.selectedWorkspacePk;
  },
  { immediate: true },
);

const updateRoute = () => {
  window.location.href = `${import.meta.env.VITE_AUTH_API_URL}/workspaces/${
    selectedWorkspacePk.value
  }/go`;
};

const workspaceDropdownOptions = computed(() =>
  _.map(workspacesStore.allWorkspaces ?? [], (w) => ({
    value: w.pk,
    label: w.displayName,
  })),
);
</script>
