<template>
  <TabGroupItem :label="label" :slug="slug">
    <CodeViewer v-if="data" class="pl-3xs h-full" height="100%" :code="data" />
    <EmptyStateCard
      v-else
      iconName="no-changes"
      :primaryText="`No ${label} Available`"
      :secondaryText="`This action did not generate${emptyStateSecondaryTextNeedsAnA ? ' a' : ''} ${label}.`"
    />
  </TabGroupItem>
</template>

<script lang="ts" setup>
import { TabGroupItem } from "@si/vue-lib/design-system";
import CodeViewer from "./CodeViewer.vue";
import EmptyStateCard from "./EmptyStateCard.vue";

defineProps<{
  label: string;
  emptyStateSecondaryTextNeedsAnA?: boolean;
  data: string | undefined;
  slug: string;
}>();
</script>
