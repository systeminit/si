<template>
  <div
    :class="
      clsx(
        'component tile',
        'cursor-pointer border border-neutral-500 rounded overflow-hidden',
        themeClasses('bg-shade-0', 'bg-neutral-900'),
      )
    "
  >
    <header
      :class="clsx('p-xs', themeClasses('bg-neutral-200', 'bg-neutral-800'))"
    >
      <Icon name="logo-aws" size="lg" />
      <h2>
        <TruncateWithTooltip class="pb-xs">{{
          component.name
        }}</TruncateWithTooltip>
      </h2>
      <h3>
        <TruncateWithTooltip class="pb-xs">{{
          component.schemaName
        }}</TruncateWithTooltip>
      </h3>
      <Icon name="none" size="lg" />
    </header>
    <ol
      class="[&>li]:p-xs [&>li]:flex [&>li]:flex-row [&>li]:items-center [&>li]:gap-xs [&_.pillcounter]:w-5 [&_.pillcounter]:h-5"
    >
      <li>
        <StatusIndicatorIcon
          type="qualification"
          size="sm"
          :status="qualificationSummary"
        />
        <div>Qualifications</div>
        <PillCounter
          :count="component.qualificationTotals.total"
          size="sm"
          class="ml-auto"
        />
      </li>
      <li>
        <Icon name="tilde" class="text-warning-500" size="sm" />
        <div>Diff</div>
        <PillCounter :count="component.diffCount" size="sm" class="ml-auto" />
      </li>
      <li>
        <StatusIndicatorIcon
          type="resource"
          size="sm"
          :status="component.hasResource ? 'exists' : 'notexists'"
        />
        <div>Resource</div>
      </li>
      <hr class="border-neutral-500" />
      <li>
        <Icon name="output-connection" size="sm" />
        <div>Inputs</div>
        <PillCounter :count="component.inputCount" size="sm" class="ml-auto" />
      </li>
      <li>
        <Icon name="input-connection" size="sm" />
        <div>Outputs</div>
        <PillCounter :count="0" size="sm" class="ml-auto" />
      </li>
    </ol>
    <!-- <footer class="grid grid-cols-2 p-xs">
      <div class="place-self-start">
        <VButton label="📌" size="sm" tone="neutral" variant="ghost" />
        <VButton label="Upgrade" size="sm" tone="action" />
      </div>
      <div class="place-self-end">
        <VButton label="Delete" size="sm" tone="destructive" />
        <VButton label="Edit" size="sm" tone="action" />
      </div>
    </footer> -->
  </div>
</template>

<script lang="ts" setup>
import {
  Icon,
  PillCounter,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed } from "vue";
import { BifrostComponent } from "@/workers/types/dbinterface";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";

const props = defineProps<{
  component: BifrostComponent;
}>();

const qualificationSummary = computed(() => {
  if (props.component.qualificationTotals.failed > 0) return "failure";
  if (props.component.qualificationTotals.running > 0) return "running";
  if (props.component.qualificationTotals.warned > 0) return "warning";
  return "success";
});
</script>
