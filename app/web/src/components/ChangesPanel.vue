<template>
  <TabGroup
    rememberSelectedTabKey="proposed_right"
    trackingSlug="actions_applied"
    variant="minimal"
    growTabsToFillWidth
    firstTabMarginLeft="none"
    marginTop="2xs"
  >
    <TabGroupItem label="Proposed" slug="actions_proposed">
      <ChangesPanelProposed />
    </TabGroupItem>

    <TabGroupItem label="Applied" slug="actions_applied">
      <ApplyHistory />
    </TabGroupItem>
  </TabGroup>
</template>

<script lang="ts" setup>
import { TabGroup, TabGroupItem } from "@si/vue-lib/design-system";
import ApplyHistory from "./ApplyHistory.vue";
import ChangesPanelProposed from "./ChangesPanelProposed.vue";
</script>
