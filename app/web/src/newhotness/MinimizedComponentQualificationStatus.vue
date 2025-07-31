<template>
  <div class="flex flex-row justify-between">
    <div class="flex flex-row items-center gap-xs">
      <template v-if="!noText">
        <TextPill
          v-if="
            component.qualificationTotals.failed === 0 &&
            component.qualificationTotals.warned === 0 &&
            component.qualificationTotals.succeeded === 0
          "
          tighter
          :class="
            clsx(
              'text-sm ml-auto',
              themeClasses(
                'border-neutral-500 bg-neutral-100 text-black',
                'border-neutral-600 bg-neutral-900 text-white',
              ),
            )
          "
        >
          Unknown
        </TextPill>
        <TextPill
          v-else-if="
            component.qualificationTotals.failed === 0 &&
            component.qualificationTotals.warned === 0
          "
          tighter
          :class="
            clsx(
              'text-xs ml-auto',
              themeClasses(
                'border-success-500 bg-neutral-100 text-black',
                'border-success-600 bg-neutral-900 text-white',
              ),
            )
          "
        >
          All passed
        </TextPill>
        <TextPill
          v-else-if="component.qualificationTotals.failed > 0"
          tighter
          :class="
            clsx(
              'text-xs ml-auto',
              themeClasses(
                'border-destructive-500 bg-destructive-100 text-black',
                'border-destructive-600 bg-destructive-900 text-white',
              ),
            )
          "
        >
          {{ component.qualificationTotals.failed }} Failed
        </TextPill>
        <TextPill
          v-else-if="component.qualificationTotals.warned > 0"
          tighter
          :class="
            clsx(
              'text-xs ml-auto',
              themeClasses(
                'border-warning-500 bg-warning-100 text-black',
                'border-warning-600 bg-warning-900 text-white',
              ),
            )
          "
        >
          {{ component.qualificationTotals.warned }} Warning
        </TextPill>
      </template>
      <StatusIndicatorIcon
        type="qualification"
        size="sm"
        :status="qualificationStatus"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { themeClasses, TextPill } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed } from "vue";
import {
  ComponentInList,
  BifrostComponent,
} from "@/workers/types/entity_kind_types";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";

const props = defineProps<{
  component: ComponentInList | BifrostComponent;
  noText?: boolean;
}>();

const qualificationStatus = computed(() => {
  if (props.component.qualificationTotals.failed > 0) return "failure";
  if (props.component.qualificationTotals.warned > 0) return "warning";
  if (props.component.qualificationTotals.succeeded > 0) return "success";
  return "unknown";
});
</script>
