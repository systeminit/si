<template>
  <div class="flex flex-row items-stretch">
    <RouterLink
      v-if="!compact && !hideEditButton && featureFlagsStore.EDIT_WORKSPACES"
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
      @mousedown="tracker.trackEvent('workspace_launcher_widget_click')"
    >
      <Icon v-if="!compact" name="laptop" size="lg" />
      <Stack spacing="xs" class="overflow-hidden">
        <div
          ref="workspaceNameRef"
          v-tooltip="workspaceNameTooltip"
          class="font-bold line-clamp-3 break-words pb-[2px]"
        >
          {{ workspace.displayName }}
        </div>
        <div class="text-sm opacity-70 capsize">
          <div class="truncate w-full">{{ workspace.instanceUrl }}</div>
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
        <div class="flex items-center text-xs gap-md pt-xs">
          <div class="flex items-center gap-xs">
            <div
              :class="
                clsx(
                  'w-[12px] h-[12px] rounded-full animate-ping absolute',
                  onboardingStore.devFrontendOnline
                    ? 'bg-success-500'
                    : 'bg-destructive-500',
                )
              "
            />
            <div
              :class="
                clsx(
                  'w-[12px] h-[12px] rounded-full shadow-lg',
                  onboardingStore.devFrontendOnline
                    ? 'bg-success-500'
                    : 'bg-destructive-500',
                )
              "
            />
            <div class="capsize">
              Frontend
              <template v-if="!compact">
                {{ onboardingStore.devFrontendOnline ? "online" : "offline" }}
              </template>
            </div>
          </div>
          <div class="flex items-center gap-xs">
            <div
              :class="
                clsx(
                  'w-[12px] h-[12px] rounded-full animate-ping absolute',
                  onboardingStore.devBackendOnline
                    ? 'bg-success-500'
                    : 'bg-destructive-500',
                )
              "
            />
            <div
              :class="
                clsx(
                  'w-[12px] h-[12px] rounded-full shadow-lg',
                  onboardingStore.devBackendOnline
                    ? 'bg-success-500'
                    : 'bg-destructive-500',
                )
              "
            />
            <div class="capsize">
              Backend
              <template v-if="!compact">
                {{ onboardingStore.devBackendOnline ? "online" : "offline" }}
              </template>
            </div>
          </div>
        </div>
      </Stack>
      <div class="ml-auto">
        <Icon
          v-if="compact"
          name="external-link"
          :class="clsx(!onboardingStore.devInstanceOnline && 'opacity-30')"
        />
        <Icon
          v-else
          name="arrow--right"
          size="lg"
          :class="
            clsx(
              onboardingStore.devInstanceOnline && 'group-hover:translate-x-1',
              !onboardingStore.devInstanceOnline && 'opacity-30',
            )
          "
        />
      </div>
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
import { useOnboardingStore } from "@/store/onboarding.store";

import { API_HTTP_URL } from "@/store/api";
import { tracker } from "@/lib/posthog";

const featureFlagsStore = useFeatureFlagsStore();

const props = defineProps({
  workspaceId: { type: String as PropType<WorkspaceId> },
  compact: Boolean,
  hideEditButton: Boolean,
});

const onboardingStore = useOnboardingStore();

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
</script>
