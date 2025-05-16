<template>
  <SourceTooltip
    :icon="icon"
    :justMessage="justMessage"
    :overridden="overridden"
    :tooltipText="tooltipText"
  >
    <SourceIcon :header="header" :icon="icon" :overridden="overridden" />
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
  justMessage: { type: Boolean },
});
</script>
