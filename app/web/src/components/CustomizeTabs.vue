<template>
  <div class="relative flex-grow">
    <TabGroup
      :start-selected-tab-slug="tabContentSlug"
      @update:selected-tab="onTabChange"
    >
      <TabGroupItem slug="functions" label="FUNCTIONS">
        <slot v-if="tabContentSlug === 'functions'" />
      </TabGroupItem>
      <TabGroupItem slug="packages" label="MODULES">
        <slot v-if="tabContentSlug === 'packages'" />
      </TabGroupItem>
      <TabGroupItem slug="assets" label="ASSETS">
        <slot v-if="tabContentSlug === 'assets'" />
      </TabGroupItem>
    </TabGroup>
  </div>
</template>

<script lang="ts" setup>
import { useRouter, useRoute } from "vue-router";
import { PropType } from "vue";
import { TabGroup, TabGroupItem } from "@si/vue-lib/design-system";

const router = useRouter();
const route = useRoute();

defineProps({
  tabContentSlug: {
    type: String as PropType<"functions" | "packages" | "assets">,
    required: true,
  },
});

function onTabChange(tabSlug?: string) {
  if (tabSlug && route.name !== `workspace-lab-${tabSlug}`) {
    router.push({ name: `workspace-lab-${tabSlug}` });
  }
}
</script>
