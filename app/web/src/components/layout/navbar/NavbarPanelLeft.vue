<template>
  <div class="flex flex-1 items-center">
    <SiLogo class="block h-[44px] w-[44px] ml-[12px] mr-[12px] flex-none" />

    <label>
      <div
        class="text-[11px] mt-[1px] mb-[5px] capsize font-medium text-neutral-300"
      >
        WORKSPACE:
      </div>
      <VormInput
        v-model="selectedWorkspacePk"
        class="flex-grow font-bold"
        size="xs"
        type="dropdown"
        noLabel
        :options="workspaceDropdownOptions"
        placeholder="-- select a workspace --"
        @change="updateRoute"
      />
    </label>

    <Icon name="chevron--right" size="xs" tone="neutral" class="mt-[14px]" />

    <ChangeSetPanel class="max-w-[50%]" />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import { Icon, VormInput } from "@si/vue-lib/design-system";
import { computed, ref, watch } from "vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
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
