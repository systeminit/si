<template>
  <div
    class="flex flex-row flex-1 basis-1/2 items-center min-w-[340px] h-full overflow-hidden"
  >
    <SiLogo class="block h-[44px] w-[44px] ml-[12px] mr-[12px] flex-none" />

    <label class="flex flex-col flex-1 min-w-0 max-w-fit">
      <div
        class="text-[11px] mt-[1px] mb-[5px] capsize font-medium text-neutral-300"
      >
        WORKSPACE:
      </div>
      <DropdownMenuButton
        ref="dropdownMenuRef"
        v-model="selectedWorkspaceId"
        :options="searchFilteredWorkspaceDropdownOptions"
        :search="
          workspaceDropdownOptions.length > DEFAULT_DROPDOWN_SEARCH_THRESHOLD
        "
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

    <Icon
      name="chevron--right"
      size="xs"
      tone="neutral"
      class="mt-[14px] flex-none"
    />

    <ChangeSetPanel ref="changeSetPanelRef" :changeSetId="changeSetId" />

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
import { computed, ref, watch } from "vue";
import { useQuery } from "@tanstack/vue-query";
import StatusPanel from "@/newhotness/StatusPanel.vue";
import ChangeSetPanel from "./ChangeSetPanel.vue";
import { routes, useApi } from "../api_composables";

export type WorkspacePk = string;

type InstanceEnvType = "LOCAL" | "PRIVATE" | "SI";

type AuthApiWorkspace = {
  creatorUserId: string;
  displayName: string;
  id: WorkspacePk;
  pk: WorkspacePk; // not actually in the response, but we backfill
  instanceEnvType: InstanceEnvType;
  instanceUrl: string;
  role: "OWNER" | "EDITOR";
  token: string;
  isHidden: boolean;
  approvalsEnabled: boolean;
};

const props = defineProps<{
  workspaceId: string;
  changeSetId: string;
}>();

const selectedWorkspaceId = ref(props.workspaceId);
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

  window.location.href = `${
    import.meta.env.VITE_AUTH_API_URL
  }/workspaces/${newWorkspacePk}/go`;
};

const workspaceApi = useApi();
const workspaceQuery = useQuery<Record<string, AuthApiWorkspace>>({
  queryKey: ["workspaces"],
  staleTime: 5000,
  queryFn: async () => {
    const call = workspaceApi.endpoint<AuthApiWorkspace[]>(routes.Workspaces);
    const response = await call.get();
    if (workspaceApi.ok(response)) {
      const renameIdList = _.map(response.data, (w) => ({
        ...w,
        pk: w.id,
      }));
      const workspacesByPk = _.keyBy(renameIdList, "pk");
      return workspacesByPk;
    }
    return {} as Record<string, AuthApiWorkspace>;
  },
});

const workspaceDropdownOptions = computed(() =>
  _.map(
    _.filter(workspaceQuery.data.value, (w) => !w.isHidden),
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
