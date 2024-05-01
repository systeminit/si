<template>
  <ScrollArea>
    <template #top>
      <SidebarSubpanelTitle
        :label="
          changeSetStore.headSelected
            ? 'HEAD'
            : changeSetStore.selectedChangeSet?.name
        "
        :icon="changeSetStore.headSelected ? 'git-branch' : 'git-branch'"
      />

      <div
        v-if="!changeSetStore.headSelected"
        :class="
          clsx(
            'flex flex-row items-center justify-center text-neutral-400 gap-xs p-xs border-b shrink-0',
            themeClasses('border-neutral-200', 'border-neutral-600'),
          )
        "
      >
        <ApplyChangeSetButton class="grow" />
        <PillCounter
          tone="action"
          :count="1 + _.keys(actionsStore.proposedActions).length"
          size="xl"
          class="bg-action-100 dark:bg-action-800 px-3 font-bold"
        />
      </div>
    </template>

    <div class="absolute inset-0">
      <TabGroup
        startSelectedTabSlug="changes"
        marginTop="2xs"
        rememberSelectedTabKey="no-selection-details-panel"
        trackingSlug="no-selection-details-panel"
      >
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
import {
  TabGroup,
  TabGroupItem,
  themeClasses,
  ScrollArea,
  PillCounter,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useActionsStore } from "@/store/actions.store";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";
import ChangesPanel from "./ChangesPanel.vue";
import SecretsPanel from "./SecretsPanel.vue";

const changeSetStore = useChangeSetsStore();
const actionsStore = useActionsStore();
</script>
