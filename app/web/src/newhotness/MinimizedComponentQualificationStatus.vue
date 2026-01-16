<template>
  <div class="flex flex-row justify-between">
    <div class="flex flex-row items-center gap-xs">
      <template v-if="!noText && !inProgress">
        <TextPill
          v-if="
            component.qualificationTotals.failed === 0 &&
            component.qualificationTotals.warned === 0 &&
            component.qualificationTotals.succeeded === 0
          "
          tighter
          variant="component"
          class="text-xs ml-auto"
        >
          Unknown
        </TextPill>
        <TextPill
          v-else-if="component.qualificationTotals.failed === 0 && component.qualificationTotals.warned === 0"
          tighter
          variant="success"
          class="text-xs ml-auto"
        >
          All passed
        </TextPill>
        <TextPill
          v-else-if="component.qualificationTotals.failed > 0"
          tighter
          variant="destructive"
          class="text-xs ml-auto"
        >
          {{ component.qualificationTotals.failed }} Failed
        </TextPill>
        <TextPill
          v-else-if="component.qualificationTotals.warned > 0"
          tighter
          variant="warning"
          class="text-xs ml-auto"
        >
          {{ component.qualificationTotals.warned }} Warning
        </TextPill>
      </template>
      <StatusIndicatorIcon type="qualification" size="sm" :status="qualificationStatus" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { TextPill } from "@si/vue-lib/design-system";
import { computed } from "vue";
import { ComponentInList, BifrostComponent, DependentValues } from "@/workers/types/entity_kind_types";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";

const props = defineProps<{
  component: ComponentInList | BifrostComponent;
  dependentValues?: DependentValues | null;
  noText?: boolean;
}>();

const inProgress = computed(() =>
  props.dependentValues?.componentAttributes?.[props.component.id]?.includes("/qualification"),
);

const qualificationStatus = computed(() => {
  if (inProgress.value) return "running";
  if (props.component.qualificationTotals.failed > 0) return "failure";
  if (props.component.qualificationTotals.warned > 0) return "warning";
  if (props.component.qualificationTotals.succeeded > 0) return "success";
  return "unknown";
});
</script>
