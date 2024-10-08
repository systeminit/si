<template>
  <div class="flex flex-row items-stretch">
    <RouterLink
      v-if="
        !compact &&
        !hideEditButton &&
        featureFlagsStore.EDIT_WORKSPACES &&
        workspace
      "
      :to="{
        name: 'workspace-settings',
        params: { workspaceId: workspace.id },
      }"
      :class="
        clsx(
          'flex-none flex flex-row items-center rounded-tl-md rounded-bl-md z-10 cursor-pointer text-shade-0',
          'mr-[-0.5rem] transition-all',
          'bg-neutral-400 dark:bg-neutral-600 hover:bg-neutral-500 dark:hover:bg-neutral-500 hover:p-sm hover:pr-md p-xs pr-sm',
        )
      "
    >
      <Icon name="settings" size="lg" />
    </RouterLink>
    <a
      v-if="workspace"
      :href="`${API_HTTP_URL}/workspaces/${workspace.id}/go`"
      target="_blank"
      :class="
        clsx(
          'group',
          'p-sm flex items-center gap-sm rounded-md flex-grow min-w-0 overflow-hidden',
          'bg-action-600 hover:bg-action-500 text-white',
          'cursor-pointer z-20',
        )
      "
      @click="clickHandler"
      @mousedown="tracker.trackEvent('workspace_launcher_widget_click')"
    >
      <div v-if="!compact" class="flex flex-col items-center gap-1">
        <div
          v-tooltip="
            workspace.instanceEnvType === 'SI'
              ? 'Managed by System Initiative'
              : workspace.instanceEnvType === 'PRIVATE'
              ? 'Private Instance'
              : 'Local Instance'
          "
        >
          <Icon
            v-if="!compact"
            :name="workspace.instanceEnvType === 'SI' ? 'cloud' : 'laptop'"
            size="lg"
          />
        </div>
        <div v-tooltip="workspace.isFavourite ? 'Favourite Workspace' : ''">
          <Icon
            :name="workspace.isFavourite ? 'star' : 'starOutline'"
            size="lg"
          />
        </div>
      </div>
      <Stack spacing="xs" class="overflow-hidden">
        <div
          ref="workspaceNameRef"
          v-tooltip="workspaceNameTooltip"
          class="font-bold line-clamp-3 break-words pb-[2px]"
        >
          {{ workspace.displayName }}
        </div>
        <div
          v-if="workspace.instanceEnvType === 'PRIVATE'"
          class="text-sm opacity-70 capsize"
        >
          <div class="truncate w-full">
            {{ workspace.instanceUrl }}
          </div>
        </div>
        <div class="text-sm opacity-70 capsize">
          <div class="truncate w-full">
            {{ workspace.description }}
          </div>
        </div>
        <div
          v-if="workspace.role !== 'OWNER'"
          class="font-bold text-sm capsize"
        >
          Owner: {{ workspace.creatorUser.firstName }}
          {{ workspace.creatorUser.lastName }}
        </div>
        <div v-if="workspace.role !== 'OWNER'" class="text-xs">
          Invited: {{ formatters.timeAgo(workspace.invitedAt) }}
        </div>
        <div class="font-bold">Role: {{ toSentenceCase(workspace.role) }}</div>
      </Stack>
    </a>
  </div>
</template>

<script setup lang="ts">
import { computed, PropType, ref } from "vue";
import { formatters } from "@si/vue-lib";
import { Icon, Stack } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useWorkspacesStore, WorkspaceId } from "@/store/workspaces.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

import { API_HTTP_URL } from "@/store/api";
import { tracker } from "@/lib/posthog";
import { useAuthStore } from "@/store/auth.store";

const featureFlagsStore = useFeatureFlagsStore();

const props = defineProps({
  workspaceId: { type: String as PropType<WorkspaceId> },
  compact: Boolean,
  hideEditButton: Boolean,
});

function toSentenceCase(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1).toLowerCase();
}

const workspacesStore = useWorkspacesStore();
const workspace = computed(() =>
  props.workspaceId
    ? workspacesStore.workspacesById[props.workspaceId]
    : workspacesStore.defaultWorkspace,
);

const workspaceNameRef = ref();
const workspaceNameTooltip = computed(() => {
  if (
    workspaceNameRef.value &&
    workspaceNameRef.value.scrollHeight > workspaceNameRef.value.offsetHeight
  ) {
    return {
      content: workspaceNameRef.value.textContent,
      delay: { show: 700, hide: 10 },
    };
  } else {
    return {};
  }
});

const emit = defineEmits<{
  (e: "edit"): void;
}>();

const authStore = useAuthStore();
function clickHandler(e: MouseEvent) {
  if (authStore.user && !authStore.user.emailVerified) {
    // eslint-disable-next-line no-alert
    alert("You must verify your email before you can log into a workspace");
    e.preventDefault();
  }
}
</script>
