<template>
  <SourceTooltip
    :tooltipText="tooltipText"
    :overridden="overridden"
    :icon="icon"
  >
    <SourceIcon :icon="icon" :overridden="overridden" :header="header" />
  </SourceTooltip>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import { IconNames } from "@si/vue-lib/design-system";
import SourceIcon from "./SourceIcon.vue";
import SourceTooltip from "./SourceTooltip.vue";

defineProps({
  tooltipText: { type: String },
  overridden: { type: Boolean },
  icon: { type: String as PropType<IconNames>, required: true },
  header: { type: Boolean },
});
</script>
