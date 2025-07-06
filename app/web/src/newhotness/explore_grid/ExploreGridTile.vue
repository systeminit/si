<template>
  <div
    :class="
      clsx(
        'component tile',
        'cursor-pointer border rounded overflow-hidden relative select-none',
        selected || focused
          ? themeClasses('border-action-500', 'border-action-300')
          : hasFailedActions
          ? themeClasses('border-destructive-500', 'border-destructive-400')
          : [
              hovered
                ? themeClasses('border-black', 'border-white')
                : themeClasses('border-neutral-400', 'border-neutral-600'),
            ],
        themeClasses('bg-shade-0', 'bg-neutral-900'),
        component.toDelete && 'opacity-70',
      )
    "
  >
    <header
      :class="
        clsx(
          'p-xs pr-sm',
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
      <Icon v-if="component.toDelete" name="hourglass" size="md" />
      <Icon
        v-else-if="canBeUpgraded"
        name="bolt-outline"
        size="md"
        :class="
          clsx(themeClasses('text-success-500', 'text-success-400'), 'mt-[5px]')
        "
      />
    </header>
    <ol
      class="[&>li]:p-xs [&>li]:flex [&>li]:flex-row [&>li]:items-center [&>li]:gap-xs [&>li]:h-9 [&_.pillcounter]:w-5 [&_.pillcounter]:h-5"
    >
      <li>
        <!-- We need the "grow" here so that the qualification status expands fully. -->
        <ComponentQualificationStatus class="grow" :component="component" />
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
    <div
      v-if="showSelectionCheckbox"
      :class="
        clsx(
          'absolute top-2xs right-2xs border w-sm h-sm',
          selected
            ? themeClasses('border-action-500', 'border-action-300')
            : themeClasses(
                'border-neutral-400 hover:border-black',
                'border-neutral-600 hover:border-white',
              ),
        )
      "
      @click.stop.prevent.left="toggleSelection"
      @click.stop.prevent.right="toggleSelection"
    >
      <Icon
        v-if="selected"
        name="check"
        size="xs"
        :class="
          clsx(
            'absolute top-[-1px] left-[-1px]',
            themeClasses('text-black', 'text-white'),
          )
        "
      />
    </div>
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
import { computed, inject } from "vue";
import {
  BifrostComponent,
  ComponentInList,
} from "@/workers/types/entity_kind_types";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { getAssetIcon } from "../util";
import { assertIsDefined, Context, ExploreContext } from "../types";
import ComponentQualificationStatus from "../ComponentQualificationStatus.vue";

const props = defineProps<{
  component: BifrostComponent | ComponentInList;
  selected?: boolean;
  focused?: boolean;
  hovered?: boolean;
  hideConnections?: boolean;
  showSelectionCheckbox?: boolean;
  hasFailedActions?: boolean;
}>();

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const explore = inject<ExploreContext>("EXPLORE_CONTEXT");
assertIsDefined<ExploreContext>(explore);

const outgoing = computed(
  () => ctx.outgoingCounts.value[props.component.id] ?? 0,
);

const canBeUpgraded = computed(() =>
  explore.upgradeableComponents.value.has(props.component.id),
);

const toggleSelection = () => {
  if (props.selected) {
    emit("deselect");
  } else {
    emit("select");
  }
};

const emit = defineEmits<{
  (e: "select"): void;
  (e: "deselect"): void;
}>();
</script>

<script lang="ts">
// Grid tiles need to have a fixed height - make sure this number matches its total height!
export const GRID_TILE_HEIGHT = 233;

export function getQualificationStatus(
  component: BifrostComponent | ComponentInList,
) {
  if (component.qualificationTotals.failed > 0) return "failure";
  if (component.qualificationTotals.warned > 0) return "warning";
  if (component.qualificationTotals.succeeded > 0) return "success";
  return "unknown";
}
</script>
