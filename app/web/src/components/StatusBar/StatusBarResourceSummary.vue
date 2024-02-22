<template>
  <StatusBarTab :selected="selected">
    <template #icon>
      <StatusIndicatorIcon type="resource" status="exists" />
    </template>
    <template #name> Resources </template>
    <template v-if="resourceCount" #summary>
      <StatusBarTabPill v-if="resourceCount" class="border-white">
        Total:
        <b class="ml-1">{{ resourceCount }}</b>
      </StatusBarTabPill>
    </template>
  </StatusBarTab>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { useComponentsStore } from "@/store/components.store";
import StatusBarTabPill from "./StatusBarTabPill.vue";
import StatusBarTab from "./StatusBarTab.vue";

const props = defineProps({
  selected: Boolean,
});

const componentsStore = useComponentsStore();
const resourceCount = computed(
  () => componentsStore.allComponents.filter((x) => x.hasResource).length,
);
</script>
