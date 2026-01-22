<template>
  <div class="flex flex-row flex-1 basis-1/2 items-center min-w-[340px] h-full overflow-hidden">
    <SiLogo
      class="block h-[44px] w-[44px] ml-[12px] mr-[12px] flex-none cursor-pointer"
      @click="() => router.push(compositionLink)"
    />

    <label class="flex flex-col flex-1 min-w-0 max-w-fit">
      <div class="text-[11px] mt-[1px] mb-[5px] capsize font-medium text-neutral-300">WORKSPACE:</div>
      <DropdownMenuButton
        ref="dropdownMenuRef"
        v-model="selectedWorkspaceId"
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

    <template v-if="!invalidWorkspace && ctx?.queriesEnabled.value">
      <Icon name="chevron--right" size="xs" tone="neutral" class="mt-[14px] flex-none" />

      <ChangeSetPanel ref="changeSetPanelRef" :workspaceId="props.workspaceId" :changeSetId="changeSetId" />
    </template>

    <StatusPanel />
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
import { computed, inject, ref, watch } from "vue";
import StatusPanel from "@/newhotness/StatusPanel.vue";
import ChangeSetPanel from "./ChangeSetPanel.vue";
import { Context, Workspaces } from "../types";
import { RouteLocationAsPathGeneric, RouteLocationAsRelativeGeneric, useRouter } from "vue-router";

const router = useRouter();
const workspaces = inject<Workspaces>("WORKSPACES");
const ctx = inject<Context>("CONTEXT");

const props = defineProps<{
  workspaceId: string;
  changeSetId: string;
  invalidWorkspace?: boolean;
}>();

const compositionLink = computed(() => {
  const name = "new-hotness";
  const params = { workspacePk: props.workspaceId, changeSetId: props.changeSetId };
  return {
    name,
    params,
  } as RouteLocationAsRelativeGeneric | RouteLocationAsPathGeneric;
});

const selectedWorkspaceId = ref(props.invalidWorkspace ? undefined : props.workspaceId);
watch(
  () => props.workspaceId,
  () => {
    selectedWorkspaceId.value = props.workspaceId;
  },
);

const dropdownMenuRef = ref<InstanceType<typeof DropdownMenuButton>>();
const changeSetPanelRef = ref<InstanceType<typeof ChangeSetPanel>>();

const updateRoute = (newWorkspacePk: string) => {
  if (props.workspaceId === newWorkspacePk) return;

  window.location.href = `${import.meta.env.VITE_AUTH_API_URL}/workspaces/${newWorkspacePk}/go`;
};

const workspaceDropdownOptions = computed<Array<{ value: string; label: string }>>(() => {
  if (!workspaces?.workspaces?.value) return [];
  return _.map(
    _.filter(workspaces.workspaces.value, (w) => !w.isHidden),
    (w) => ({
      value: w.pk,
      label: w.displayName,
    }),
  );
});

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
