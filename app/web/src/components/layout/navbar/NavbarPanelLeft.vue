<template>
  <div class="flex flex-row flex-1 basis-1/2 items-center min-w-[340px] h-full overflow-hidden">
    <SiLogo class="block h-[44px] w-[44px] ml-[12px] mr-[12px] flex-none" />

    <label class="flex flex-col flex-1 min-w-0 max-w-fit">
      <div class="text-[11px] mt-[1px] mb-[5px] capsize font-medium text-neutral-300">WORKSPACE:</div>
      <DropdownMenuButton
        ref="dropdownMenuRef"
        v-model="selectedWorkspacePk"
        :options="searchFilteredWorkspaceDropdownOptions"
        :search="workspaceDropdownOptions.length > DEFAULT_DROPDOWN_SEARCH_THRESHOLD"
        placeholder="-- select a workspace --"
        checkable
        variant="navbar"
        @select="updateRoute"
      >
        <DropdownMenuItem
          v-if="searchFilteredWorkspaceDropdownOptions.length === 0"
          label="No Workspaces Match Your Search"
          header
        />
      </DropdownMenuButton>
    </label>

    <template v-if="!invalidWorkspace">
      <Icon name="chevron--right" size="xs" tone="neutral" class="mt-[14px] flex-none" />

      <ChangeSetPanel ref="changeSetPanelRef" />
    </template>

    <StatusPanel v-if="useNewUI" />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import {
  DEFAULT_DROPDOWN_SEARCH_THRESHOLD,
  DropdownMenuButton,
  DropdownMenuItem,
  Icon,
} from "@si/vue-lib/design-system";
import { computed, ref, watch } from "vue";
import { useRoute } from "vue-router";
import { useWorkspacesStore } from "@/store/workspaces.store";
import StatusPanel from "@/newhotness/StatusPanel.vue";
import ChangeSetPanel from "./ChangeSetPanel.vue";

defineProps({
  invalidWorkspace: { type: Boolean },
});

// Determine if we're in the new experience
const route = useRoute();
const useNewUI = computed(() => {
  return route.name?.toString().startsWith("new-hotness");
});

const workspacesStore = useWorkspacesStore();

const dropdownMenuRef = ref<InstanceType<typeof DropdownMenuButton>>();
const changeSetPanelRef = ref<InstanceType<typeof ChangeSetPanel>>();

const selectedWorkspacePk = ref<string | null>(null);
watch(
  () => workspacesStore.selectedWorkspacePk,
  () => {
    selectedWorkspacePk.value = workspacesStore.selectedWorkspacePk;
  },
  { immediate: true },
);

const updateRoute = (newWorkspacePk: string) => {
  if (selectedWorkspacePk.value === newWorkspacePk) return;

  selectedWorkspacePk.value = newWorkspacePk;
  window.location.href = `${import.meta.env.VITE_AUTH_API_URL}/workspaces/${selectedWorkspacePk.value}/go`;
};

const workspaceDropdownOptions = computed(() =>
  _.map(
    _.filter(workspacesStore.allWorkspaces ?? [], (w) => !w.isHidden),
    (w) => ({
      value: w.pk,
      label: w.displayName,
    }),
  ),
);

const searchFilteredWorkspaceDropdownOptions = computed(() => {
  const searchString = dropdownMenuRef.value?.searchString;

  if (!searchString || searchString === "") {
    return workspaceDropdownOptions.value;
  }

  return workspaceDropdownOptions.value.filter(
    (option) =>
      option.label.toLocaleLowerCase().includes(searchString) ||
      option.value.toLocaleLowerCase().includes(searchString),
  );
});

const openCreateModal = () => {
  changeSetPanelRef.value?.openCreateModal();
};
defineExpose({ openCreateModal });
</script>
