<template>
  <div
    :class="
      clsx(
        'component tile',
        'cursor-pointer border rounded overflow-hidden',
        themeClasses(
          'bg-shade-0 border-neutral-400',
          'bg-neutral-900 border-neutral-600',
        ),
        component.toDelete && 'opacity-70',
      )
    "
  >
    <header
      :class="
        clsx(
          'p-xs',
          !component.toDelete &&
            themeClasses('bg-neutral-200', 'bg-neutral-800'),
        )
      "
    >
      <Icon :name="getAssetIcon(component.schemaCategory)" size="lg" />
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
      <Icon
        v-if="canBeUpgraded"
        name="bolt-outline"
        size="lg"
        :class="clsx(themeClasses('text-success-500', 'text-success-400'))"
      />
      <!-- TODO(nick): center this vertically with the pill counters -->
      <Icon v-if="component.toDelete" name="hourglass" size="md" />
    </header>
    <ol
      class="[&>li]:p-xs [&>li]:flex [&>li]:flex-row [&>li]:items-center [&>li]:gap-xs [&>li]:h-9 [&_.pillcounter]:w-5 [&_.pillcounter]:h-5"
    >
      <li>
        <StatusIndicatorIcon
          type="qualification"
          size="sm"
          :status="qualificationSummary"
        />
        <div>Qualifications</div>
        <TextPill
          v-if="
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
          v-if="component.qualificationTotals.failed > 0"
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
          v-if="component.qualificationTotals.warned > 0"
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
      </li>
      <li>
        <template v-if="component.diffCount > 0">
          <Icon name="tilde" class="text-warning-500" size="sm" />
          <div>Diff</div>
          <PillCounter :count="component.diffCount" size="sm" class="ml-auto" />
        </template>
      </li>
      <li>
        <template v-if="component.hasResource">
          <StatusIndicatorIcon type="resource" size="sm" status="exists" />
          <div>Resource</div>
        </template>
      </li>
      <!-- NOTE: when coming from the Map page we don't have accurate outputCount, hiding this -->
      <template v-if="!props.hideConnections">
        <hr :class="themeClasses('border-neutral-400', 'border-neutral-600')" />
        <li>
          <Icon name="output-connection" size="sm" />
          <div>Incoming</div>
          <PillCounter
            :count="component.inputCount"
            size="sm"
            class="ml-auto"
          />
        </li>
        <li>
          <Icon name="input-connection" size="sm" />
          <div>Outgoing</div>
          <PillCounter :count="outgoing" size="sm" class="ml-auto" />
        </li>
      </template>
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
  TextPill,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed, inject } from "vue";
import {
  BifrostComponent,
  ComponentInList,
} from "@/workers/types/entity_kind_types";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { getAssetIcon } from "./util";
import { assertIsDefined, Context } from "./types";
import { useUpgrade } from "./logic_composables/upgrade";

const props = defineProps<{
  component: BifrostComponent | ComponentInList;
  hideConnections?: boolean;
}>();

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const outgoing = computed(
  () => ctx.outgoingCounts.value[props.component.id] ?? 0,
);

const upgrade = useUpgrade();
const schemaVariantId = computed(() => {
  if (typeof props.component.schemaVariantId === "string") {
    return props.component.schemaVariantId;
  } else {
    return props.component.schemaVariantId.id;
  }
});
const canBeUpgraded = computed(() =>
  upgrade(props.component.schemaId, schemaVariantId.value),
);

const qualificationSummary = computed(() => {
  if (props.component.qualificationTotals.failed > 0) return "failure";
  if (props.component.qualificationTotals.running > 0) return "running";
  if (props.component.qualificationTotals.warned > 0) return "warning";
  return "success";
});
</script>
