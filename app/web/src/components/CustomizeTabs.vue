<template>
  <TabGroup
    v-if="featureFlagsStore.MODULES_TAB"
    ref="group"
    :startSelectedTabSlug="tabContentSlug"
    marginTop="2xs"
    @update:selected-tab="onTabChange"
  >
    <TabGroupItem slug="assets">
      <template #label>
        <div class="flex flex-row items-center gap-xs">
          <div>Assets Installed</div>
          <PillCounter :count="assetList.length" />
        </div>
      </template>
      <slot name="assets" />
    </TabGroupItem>
    <TabGroupItem slug="packages" label="Modules (Internal)">
      <slot name="packages" />
    </TabGroupItem>
  </TabGroup>

  <!-- NOTE(nick): do not use the tab group if only one slot is used -->
  <div v-else>
    <slot name="assets" />
  </div>
</template>

<script lang="ts" setup>
import { useRouter, useRoute, RouteLocationNamedRaw } from "vue-router";
import { PropType, ref } from "vue";
import { storeToRefs } from "pinia";
import { TabGroup, TabGroupItem, PillCounter } from "@si/vue-lib/design-system";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { useAssetStore } from "@/store/asset.store";

const router = useRouter();
const route = useRoute();
const featureFlagsStore = useFeatureFlagsStore();
const assetStore = useAssetStore();

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
    if (tabSlug !== "packages") params.query = assetStore.syncSelectionIntoUrl(true);
    router.push(params);
  }
}

defineExpose({ group });
</script>
