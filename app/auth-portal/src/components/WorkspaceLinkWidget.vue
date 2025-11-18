<template>
  <div
    v-if="workspace"
    v-tooltip="{ content: workspace.instanceUrl, placement: 'bottom' }"
    :class="
      clsx(
        'flex flex-col rounded border cursor-pointer',
        themeClasses(
          'border-neutral-200 bg-shade-0 hover:border-action-500',
          'border-neutral-700 bg-neutral-800 hover:border-action-400',
        ),
      )
    "
    @click="clickHandler"
    @mousedown="tracker.trackEvent('workspace_launcher_widget_click')"
  >
    <div
      :class="
        clsx(
          'flex flex-row items-center gap-xs p-xs border-b',
          themeClasses('border-neutral-200', 'border-neutral-700'),
        )
      "
    >
      <Icon class="flex-none" name="cloud" size="sm" />
      <TruncateWithTooltip class="text-md flex-grow py-2xs">
        <a :href="`${API_HTTP_URL}/workspaces/${workspace.id}/go`">
          {{ workspace.displayName }}
        </a>
      </TruncateWithTooltip>
      <IconButton
        :icon="workspace.isFavourite ? 'star' : 'starOutline'"
        :iconIdleTone="workspace.isFavourite ? 'warning' : 'shade'"
        :tooltip="workspace.isFavourite ? 'Remove Favourite' : 'Set Favourite'"
        class="flex-none"
        iconBgActiveTone="action"
        iconTone="warning"
        size="sm"
        tooltipPlacement="top"
        @click.stop="starWorkspace"
      />
      <IconButton
        class="flex-none"
        icon="settings"
        iconIdleTone="shade"
        size="sm"
        tooltip="Settings"
        tooltipPlacement="top"
        @click.stop="dropdownMenuRef?.open($event) || _.noop"
      />
      <DropdownMenu ref="dropdownMenuRef" forceAlignRight>
        <DropdownMenuItem
          icon="settings"
          label="Settings"
          @click.stop="openSettings"
        />
        <DropdownMenuItem
          icon="key-tilted"
          label="API Tokens"
          @click.stop="openApiTokens"
        />
      </DropdownMenu>
    </div>
    <div class="flex flex-col p-xs text-xs gap-xs min-h-[80px]">
      <div class="flex flex-col justify-between text-md gap-xs">
        <div>
          <span class="font-bold">Role: </span
          >{{
            workspace.role.toLowerCase() === "editor"
              ? "collaborator"
              : workspace.role.toLowerCase()
          }}
        </div>
        <div v-if="workspace.role.toLowerCase() !== 'owner'">
          <span class="font-bold">Owner: </span
          >{{
            `${workspace.creatorUser.firstName} ${workspace.creatorUser.lastName}`
          }}
        </div>
      </div>

      <!-- TODO(Wendy) - need to pipe in the data for these pieces here
      <div class="flex flex-row justify-between">
        <div><span class="font-bold">Last Apply: </span>mm/dd/yyyy</div>
        <div><span class="font-bold">By: </span>whoever</div>
      </div> -->

      <!-- TODO(Wendy) - this too!
      <div class="flex flex-row gap-sm pt-2xs">
        <div class="flex flex-row gap-2xs items-center flex-grow">
          <Icon name="user-circle" size="sm" class="text-neutral-400" />
          <div>99</div>
        </div>
        <div class="flex flex-row gap-2xs items-center flex-none">
          <Icon
            name="check-hex"
            size="sm"
            class="text-success-600 dark:text-success-500"
          />
          <div>99</div>
        </div>
        <div class="flex flex-row gap-2xs items-center flex-none">
          <Icon
            name="x-hex-outline"
            size="sm"
            class="text-destructive-600 dark:text-destructive-500"
          />
          <div>99</div>
        </div>
      </div> -->
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType, ref } from "vue";
import {
  DropdownMenu,
  DropdownMenuItem,
  Icon,
  IconButton,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useRouter } from "vue-router";
import * as _ from "lodash-es";
import { useWorkspacesStore, WorkspaceId } from "@/store/workspaces.store";

import { API_HTTP_URL } from "@/store/api";
import { tracker } from "@/lib/posthog";
import { useAuthStore } from "@/store/auth.store";

const authStore = useAuthStore();
const router = useRouter();

const props = defineProps({
  workspaceId: { type: String as PropType<WorkspaceId> },
  compact: Boolean,
  hideEditButton: Boolean,
});

const workspacesStore = useWorkspacesStore();
const workspace = computed(() =>
  props.workspaceId
    ? workspacesStore.workspacesById[props.workspaceId]
    : undefined,
);

function clickHandler(e: MouseEvent) {
  if (authStore.user && !authStore.user.emailVerified) {
    // eslint-disable-next-line no-alert
    alert("You must verify your email before you can log into a workspace");
    e.preventDefault();
  } else {
    window.location.href = `${API_HTTP_URL}/workspaces/${props.workspaceId}/go`;
  }
}

const starWorkspace = async () => {
  if (!props.workspaceId || !workspace.value) return;

  await workspacesStore.SET_FAVOURITE(
    props.workspaceId,
    !workspace.value.isFavourite,
  );

  workspace.value.isFavourite = !workspace.value.isFavourite;
};

const dropdownMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const openSettings = async () => {
  await router.push({
    name: "workspace-settings",
    params: { workspaceId: props.workspaceId },
  });
};

const openApiTokens = async () => {
  await router.push({
    name: "workspace-api-tokens",
    params: { workspaceId: props.workspaceId },
  });
};

const emit = defineEmits<{
  (e: "edit"): void;
}>();
</script>
