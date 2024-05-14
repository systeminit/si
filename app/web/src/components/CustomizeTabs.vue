<template>
  <TabGroup
    :startSelectedTabSlug="tabContentSlug"
    marginTop="2xs"
    variant="fullsize"
    @update:selected-tab="onTabChange"
  >
    <TabGroupItem slug="assets" label="ASSETS">
      <slot v-if="tabContentSlug === 'assets'" />
    </TabGroupItem>
    <TabGroupItem slug="functions" label="FUNCTIONS">
      <slot v-if="tabContentSlug === 'functions'" />
    </TabGroupItem>
    <TabGroupItem
      v-if="featureFlagsStore.MODULES_TAB"
      slug="packages"
      label="MODULES"
    >
      <slot v-if="tabContentSlug === 'packages'" />
    </TabGroupItem>
  </TabGroup>
</template>

<script lang="ts" setup>
import { useRouter, useRoute } from "vue-router";
import { PropType } from "vue";
import { TabGroup, TabGroupItem } from "@si/vue-lib/design-system";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const router = useRouter();
const route = useRoute();
const featureFlagsStore = useFeatureFlagsStore();

defineProps({
  tabContentSlug: {
    type: String as PropType<"assets" | "functions" | "packages">,
    required: true,
  },
});

function onTabChange(tabSlug?: string) {
  if (tabSlug && route.name !== `workspace-lab-${tabSlug}`) {
    router.push({ name: `workspace-lab-${tabSlug}` });
  }
}
</script>
