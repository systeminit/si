<template>
  <TabGroup
    rememberSelectedTabKey="proposed_right"
    trackingSlug="actions_applied"
    variant="minimal"
    growTabsToFillWidth
    firstTabMarginLeft="none"
    marginTop="2xs"
  >
    <TabGroupItem
      v-if="!changeSetStore.headSelected"
      label="Proposed"
      slug="actions_proposed"
    >
      <ChangesPanelProposed />
    </TabGroupItem>

    <TabGroupItem label="Applied" slug="actions_applied">
      <ApplyHistory />
    </TabGroupItem>
  </TabGroup>
</template>

<script lang="ts" setup>
import { TabGroup, TabGroupItem } from "@si/vue-lib/design-system";
import { useChangeSetsStore } from "@/store/change_sets.store";
import ApplyHistory from "./ApplyHistory.vue";
import ChangesPanelProposed from "./ChangesPanelProposed.vue";

const changeSetStore = useChangeSetsStore();
</script>
