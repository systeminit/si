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
            'flex flex-row items-center justify-center text-neutral-400 gap-xs border-b shrink-0',
            themeClasses('border-neutral-200', 'border-neutral-600'),
          )
        "
      >
        <ApplyChangeSetButton class="grow" />
      </div>
    </template>

    <div class="absolute inset-0">
      <TabGroup
        startSelectedTabSlug="changes"
        rememberSelectedTabKey="no-selection-details-panel"
        trackingSlug="no-selection-details-panel"
        variant="fullsize"
      >
        <TabGroupItem label="CHANGES" slug="changes">
          <ChangesPanelProposed />
        </TabGroupItem>
        <TabGroupItem label="HISTORY" slug="history">
          <ChangesPanelHistory />
        </TabGroupItem>
        <TabGroupItem label="SECRETS" slug="secrets">
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
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";
import SecretsPanel from "./SecretsPanel.vue";
import ChangesPanelProposed from "./ChangesPanelProposed.vue";
import ChangesPanelHistory from "./ChangesPanelHistory.vue";

const changeSetStore = useChangeSetsStore();
</script>
./ChangesPanelHistory.vue
