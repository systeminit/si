<template>
  <VTooltip theme="attribute-source-icon">
    <SourceIcon :icon="icon" :overridden="overridden" :header="header" />
    <template #popper>
      <div class="flex flex-row items-center gap-xs">
        <SourceIcon :icon="icon" :overridden="overridden" />
        <div class="flex flex-col">
          <div>{{ tooltipText }}</div>
          <div class="text-xs italic text-neutral-400">
            {{ overridden ? "Overridden" : "Default Behavior" }}
          </div>
        </div>
      </div>
    </template>
  </VTooltip>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import { IconNames } from "@si/vue-lib/design-system";
import SourceIcon from "./SourceIcon.vue";

defineProps({
  tooltipText: { type: String },
  overridden: { type: Boolean },
  icon: { type: String as PropType<IconNames>, required: true },
  header: { type: Boolean },
});
</script>
