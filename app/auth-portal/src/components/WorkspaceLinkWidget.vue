<template>
  <div class="flex flex-row items-stretch">
    <div
      v-if="!compact && featureFlagsStore.EDIT_WORKSPACES"
      :class="
        clsx(
          'flex-none flex flex-row items-center rounded-tl-md rounded-bl-md z-10 cursor-pointer text-shade-0',
          'mr-[-0.5rem] transition-all',
          editing
            ? 'bg-neutral-500 p-sm pr-md'
            : 'bg-neutral-400 dark:bg-neutral-600 hover:bg-neutral-500 dark:hover:bg-neutral-500 hover:p-sm hover:pr-md p-xs pr-sm',
        )
      "
      @click="emit('edit')"
    >
      <Icon name="settings" size="lg" />
    </div>
    <a
      v-if="workspace"
      :href="`${API_HTTP_URL}/workspaces/${workspace.id}/go`"
      target="_blank"
      :class="
        clsx(
          'group',
          'p-sm flex items-center gap-sm rounded-md flex-grow',
          'bg-action-600 hover:bg-action-500 text-white',
          'cursor-pointer z-20',
        )
      "
      @mousedown="tracker.trackEvent('workspace_launcher_widget_click')"
    >
      <Icon v-if="!compact" name="laptop" size="lg" />
      <Stack spacing="xs">
        <div class="font-bold capsize">{{ workspace.displayName }}</div>
        <div class="text-sm opacity-70 capsize">
          {{ workspace.instanceUrl }}
        </div>
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
import { computed, PropType } from "vue";

import { Icon, Stack } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useWorkspacesStore, WorkspaceId } from "@/store/workspaces.store";
import { useOnboardingStore } from "@/store/onboarding.store";

import { API_HTTP_URL } from "@/store/api";
import { tracker } from "@/lib/posthog";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const featureFlagsStore = useFeatureFlagsStore();

const props = defineProps({
  workspaceId: { type: String as PropType<WorkspaceId> },
  compact: Boolean,
  editing: Boolean,
});

const onboardingStore = useOnboardingStore();

const workspacesStore = useWorkspacesStore();
const workspace = computed(() =>
  props.workspaceId
    ? workspacesStore.workspacesById[props.workspaceId]
    : workspacesStore.defaultWorkspace,
);

const emit = defineEmits<{
  (e: "edit"): void;
}>();
</script>
