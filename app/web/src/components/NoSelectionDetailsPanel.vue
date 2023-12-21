<template>
  <ScrollArea>
    <template #top>
      <SidebarSubpanelTitle
        :label="
          changeSetStore.headSelected
            ? 'Workspace Activity'
            : 'Change Set Details'
        "
        :icon="changeSetStore.headSelected ? 'git-branch' : 'git-branch'"
      />

      <div
        v-if="
          !changeSetStore.headSelected && componentsStore.allComponents.length
        "
        :class="
          clsx(
            'flex flex-row items-center justify-center text-neutral-400 gap-2 p-xs border-b shrink-0',
            themeClasses('border-neutral-200', 'border-neutral-600'),
          )
        "
      >
        <ApplyChangeSetButton class="grow" />
        <strong
          class="text-action-300 bg-action-100 text-lg rounded-2xl px-3 border border-action-300"
        >
          {{ 1 + diffs.length + _.keys(actionsStore.proposedActions).length }}
        </strong>
      </div>
    </template>

    <template v-if="componentsStore.allComponents.length === 0">
      <div class="flex flex-col items-center text-neutral-400 pt-lg">
        <EmptyStateIcon name="no-assets" class="mt-3" />
        <span class="text-xl dark:text-neutral-300">Your Model Is Empty</span>
        <div class="capsize px-xs py-md italic text-sm text-center">
          Drag some assets onto the diagram
        </div>
      </div>
    </template>

    <template v-else>
      <div class="absolute inset-0">
        <TabGroup startSelectedTabSlug="changes">
          <TabGroupItem label="Changes" slug="changes">
            <ChangesPanel />
          </TabGroupItem>
          <TabGroupItem
            v-if="featureFlagsStore.SECRETS_MANAGEMENT"
            label="Secrets"
            slug="secrets"
          >
            <SecretsPanel />
          </TabGroupItem>
        </TabGroup>
      </div>
    </template>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import {
  TabGroup,
  TabGroupItem,
  themeClasses,
  ScrollArea,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { useActionsStore } from "@/store/actions.store";
import EmptyStateIcon from "./EmptyStateIcon.vue";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";
import ChangesPanel from "./ChangesPanel.vue";
import SecretsPanel from "./SecretsPanel.vue";

const changeSetStore = useChangeSetsStore();
const componentsStore = useComponentsStore();
const featureFlagsStore = useFeatureFlagsStore();
const actionsStore = useActionsStore();

const diffs = computed(() => {
  const arr = Object.values(componentsStore.componentsById)
    .filter((c) => c.changeStatus !== "unmodified")
    .map((c) => {
      let updatedAt = c.updatedInfo.timestamp;
      if (c.changeStatus === "added") {
        updatedAt = c.createdInfo.timestamp;
      } else if (c.changeStatus === "deleted" && c.deletedInfo) {
        updatedAt = c.deletedInfo.timestamp;
      }

      return {
        componentId: c.id,
        status: c.changeStatus,
        updatedAt,
      };
    });
  arr.sort(
    (a, b) => new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime(),
  );
  return arr;
});
</script>
