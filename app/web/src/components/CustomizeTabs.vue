<template>
  <TabGroup
    ref="group"
    :startSelectedTabSlug="tabContentSlug"
    marginTop="2xs"
    @update:selected-tab="onTabChange"
  >
    <TabGroupItem slug="assets">
      <template #label>
        Assets Installed
        <PillCounter :count="assetList.length" />
      </template>
      <slot name="assets" />
    </TabGroupItem>
    <TabGroupItem slug="newassets">
      <template #label>
        Assets Available
        <PillCounter :count="moduleStore.installableModules.length" />
      </template>
      <slot name="newassets" />
    </TabGroupItem>
    <TabGroupItem
      v-if="featureFlagsStore.MODULES_TAB"
      slug="packages"
      label="M"
    >
      <slot name="packages" />
    </TabGroupItem>
  </TabGroup>
</template>

<script lang="ts" setup>
import { useRouter, useRoute, RouteLocationNamedRaw } from "vue-router";
import { PropType, ref } from "vue";
import { storeToRefs } from "pinia";
import { TabGroup, TabGroupItem, PillCounter } from "@si/vue-lib/design-system";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { useAssetStore } from "@/store/asset.store";
import { useModuleStore } from "@/store/module.store";

const router = useRouter();
const route = useRoute();
const featureFlagsStore = useFeatureFlagsStore();
const assetStore = useAssetStore();
const moduleStore = useModuleStore();

const { variantList: assetList } = storeToRefs(assetStore);

const group = ref<InstanceType<typeof TabGroup>>();

defineProps({
  tabContentSlug: {
    type: String as PropType<"assets" | "newassets" | "packages">,
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
    if (tabSlug !== "packages")
      params.query = assetStore.syncSelectionIntoUrl(true);
    router.push(params);
  }
}

defineExpose({ group });
</script>
