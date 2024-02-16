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
        <PillCounter
          tone="action"
          :count="
            1 + diffs.length + _.keys(actionsStore.proposedActions).length
          "
          size="xl"
          class="bg-action-100 dark:bg-action-800 px-3 font-bold"
        />
      </div>
    </template>

    <div class="absolute inset-0">
      <TabGroup startSelectedTabSlug="changes" marginTop="2xs">
        <TabGroupItem label="Changes" slug="changes">
          <ChangesPanel />
        </TabGroupItem>
        <TabGroupItem label="Secrets" slug="secrets">
          <SecretsPanel />
        </TabGroupItem>
      </TabGroup>
    </div>
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
  PillCounter,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useActionsStore } from "@/store/actions.store";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";
import ChangesPanel from "./ChangesPanel.vue";
import SecretsPanel from "./SecretsPanel.vue";

const changeSetStore = useChangeSetsStore();
const componentsStore = useComponentsStore();
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
