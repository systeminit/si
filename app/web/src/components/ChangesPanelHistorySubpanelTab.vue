<template>
  <TabGroupItem :label="label" :slug="slug">
    <CodeViewer
      v-if="data"
      class="pl-[2px] h-full"
      height="100%"
      :code="data"
    />
    <EmptyStateCard
      v-else
      iconName="no-changes"
      :primaryText="`No ${label} Available`"
      :secondaryText="`This action did not generate${
        emptyStateSecondaryTextNeedsAnA ? ' a' : ''
      } ${label}.`"
    />
  </TabGroupItem>
</template>

<script lang="ts" setup>
import { TabGroupItem } from "@si/vue-lib/design-system";
import CodeViewer from "./CodeViewer.vue";
import EmptyStateCard from "./EmptyStateCard.vue";

defineProps({
  label: { type: String, required: true },
  emptyStateSecondaryTextNeedsAnA: { type: Boolean },
  data: { type: String },
  slug: { type: String },
});
</script>
