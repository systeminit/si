<template>
  <div class="relative flex-grow">
    <TabGroup
      :start-selected-tab-slug="tabContentSlug"
      @update:selected-tab="onTabChange"
    >
      <TabGroupItem slug="functions" label="FUNCTIONS">
        <slot v-if="tabContentSlug === 'functions'" />
      </TabGroupItem>
      <TabGroupItem slug="packages" label="PACKAGES">
        <slot v-if="tabContentSlug === 'packages'" />
      </TabGroupItem>
      <TabGroupItem slug="assets" label="ASSETS">
        <slot v-if="tabContentSlug === 'assets'" />
      </TabGroupItem>
    </TabGroup>
  </div>
</template>

<script lang="ts" setup>
import { useRouter } from "vue-router";
import { PropType } from "vue";
import TabGroup from "@/ui-lib/tabs/TabGroup.vue";
import TabGroupItem from "@/ui-lib/tabs/TabGroupItem.vue";

const router = useRouter();

defineProps({
  tabContentSlug: {
    type: String as PropType<"functions" | "packages" | "assets">,
    required: true,
  },
});

function onTabChange(tabSlug?: string) {
  if (!tabSlug) {
    router.push({ name: `workspace-lab` });
  } else {
    router.push({ name: `workspace-lab-${tabSlug}` });
  }
}
</script>
