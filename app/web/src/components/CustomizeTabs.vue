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
import { useRouter, useRoute, RouteLocationNamedRaw } from "vue-router";
import { PropType } from "vue";
import { TabGroup, TabGroupItem } from "@si/vue-lib/design-system";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { useAssetStore } from "@/store/asset.store";

const router = useRouter();
const route = useRoute();
const featureFlagsStore = useFeatureFlagsStore();
const assetStore = useAssetStore();

defineProps({
  tabContentSlug: {
    type: String as PropType<"assets" | "functions" | "packages">,
    required: true,
  },
});

function onTabChange(tabSlug?: string) {
  // keep selections in the URL bar as you move to asset tab
  // NOT FOR PAGE LOAD
  if (tabSlug && route.name !== `workspace-lab-${tabSlug}`) {
    const params = {
      name: `workspace-lab-${tabSlug}`,
    } as RouteLocationNamedRaw;
    if (tabSlug === "assets")
      params.query = assetStore.syncSelectionIntoUrl(true);
    router.push(params);
  }
}
</script>
