<template>
  <div
    ref="gridTile"
    :class="
      clsx(
        'component tile',
        'cursor-pointer rounded overflow-hidden relative select-none',
        hasRunningActions ? 'spinning-border' : 'border',
        !hasRunningActions && [
          selected || focused
            ? themeClasses('border-action-500', 'border-action-300')
            : hasFailedActions
            ? themeClasses('border-destructive-500', 'border-destructive-400')
            : [
                hovered
                  ? themeClasses('border-black', 'border-white')
                  : themeClasses('border-neutral-400', 'border-neutral-600'),
              ],
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
      <Icon
        :name="getAssetIcon(component.schemaCategory)"
        size="md"
        class="place-self-center my-auto mt-1"
      />
      <h2>
        <TruncateWithTooltip class="pb-xs text-sm">
          {{ component.name }}
        </TruncateWithTooltip>
      </h2>
      <h3>
        <TruncateWithTooltip class="pb-xs text-xs">
          {{ component.schemaName }}
        </TruncateWithTooltip>
      </h3>
      <Icon v-if="component.toDelete" name="trash" size="md" />
      <Icon
        v-else-if="component.hasSocketConnections"
        v-tooltip="'Incompatibility found'"
        name="alert-triangle-filled"
        size="lg"
        :class="
          clsx(
            themeClasses('text-warning-600', 'text-warning-400'),
            'absolute top-[8px] right-[24px] rounded p-xs hover:cursor-pointer',
            themeClasses('hover:bg-neutral-300', 'hover:bg-neutral-600'),
          )
        "
      />
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
        <ComponentTileQualificationStatus class="grow" :component="component" />
      </li>

      <!-- Rows 2-4: Dynamic content based on priority -->
      <li v-for="(rowContent, index) in dynamicRows" :key="index">
        <template v-if="rowContent === 'resource'">
          <StatusIndicatorIcon
            v-if="component.resourceId"
            v-tooltip="'Resource'"
            type="resource"
            size="sm"
            status="exists"
          />
          <StatusIndicatorIcon
            v-else
            type="resource"
            size="sm"
            status="exists"
          />
          <TruncateWithTooltip
            v-if="component.resourceId"
            class="text-xs py-xs opacity-75"
          >
            {{ component.resourceId }}
          </TruncateWithTooltip>
          <div v-else class="text-sm">Resource</div>
        </template>
        <template v-else-if="rowContent === 'diff'">
          <Icon name="tilde" class="text-warning-500" size="sm" />
          <div class="text-sm">Diff</div>
          <TextPill
            v-if="component.diffStatus === ComponentDiffStatus.Added"
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
            Added
          </TextPill>
          <TextPill
            v-else-if="component.diffStatus === ComponentDiffStatus.Modified"
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
            Modified
          </TextPill>
        </template>
        <template v-else-if="rowContent === 'pending'">
          <div class="grid grid-cols-5 gap-1 items-center">
            <TextPill
              v-for="(actionData, actionName) in pendingActionCounts"
              :key="actionName"
              v-tooltip="
                getPendingActionTooltip(
                  actionName,
                  actionData.count,
                  actionData.hasFailed,
                )
              "
              variant="key2"
              size="sm"
              class="text-xs flex items-center gap-1"
            >
              <Icon
                :name="getPendingActionIcon(actionName)"
                :class="
                  getPendingActionIconClass(actionName, actionData.hasFailed)
                "
                size="xs"
              />
              {{ actionData.count }}
            </TextPill>
          </div>
        </template>
      </li>
      <!-- NOTE: when coming from the Map page we don't have accurate outputCount, hiding this -->
      <template v-if="!props.hideConnections">
        <hr :class="themeClasses('border-neutral-400', 'border-neutral-600')" />
        <li>
          <Icon name="output-connection" size="sm" />
          <div class="text-sm">Incoming</div>
          <PillCounter
            :count="component.inputCount"
            size="sm"
            class="ml-auto"
          />
        </li>
        <li>
          <Icon name="input-connection" size="sm" />
          <div class="text-sm">Outgoing</div>
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
  IconNames,
  PillCounter,
  TextPill,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed, inject, ref, watch } from "vue";
import {
  ComponentInList,
  ComponentDiffStatus,
} from "@/workers/types/entity_kind_types";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { getAssetIcon } from "../util";
import { assertIsDefined, Context, ExploreContext } from "../types";
import ComponentTileQualificationStatus from "../ComponentTileQualificationStatus.vue";

const props = defineProps<{
  component: ComponentInList;
  selected?: boolean;
  focused?: boolean;
  hovered?: boolean;
  hideConnections?: boolean;
  showSelectionCheckbox?: boolean;
  hasFailedActions?: boolean;
  hasRunningActions?: boolean;
  pendingActionCounts?: Record<string, { count: number; hasFailed: boolean }>;
}>();

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const explore = inject<ExploreContext>("EXPLORE_CONTEXT");
assertIsDefined<ExploreContext>(explore);

const outgoing = computed(
  () => ctx.outgoingCounts.value[props.component.id] ?? 0,
);

const gridTile = ref<HTMLElement | undefined>();

const canBeUpgraded = computed(() =>
  explore.upgradeableComponents.value.has(props.component.id),
);

const dynamicRows = computed(() => {
  const rows: (string | null)[] = [];
  const hasPendingActions =
    props.pendingActionCounts &&
    Object.keys(props.pendingActionCounts).length > 0;
  const hasDiff =
    props.component.diffStatus &&
    props.component.diffStatus !== ComponentDiffStatus.None;

  // Row 2: Resource if exists, otherwise diff if exists, otherwise empty
  if (props.component.hasResource) {
    rows.push("resource");
  } else if (hasDiff) {
    rows.push("diff");
  } else {
    rows.push(null);
  }

  // Row 3: Diff if exists AND resource exists, otherwise empty
  if (props.component.hasResource && hasDiff) {
    rows.push("diff");
  } else {
    rows.push(null);
  }

  // Row 4: Pending actions if they exist, otherwise empty
  if (hasPendingActions) {
    rows.push("pending");
  } else {
    rows.push(null);
  }

  return rows;
});
const toggleSelection = () => {
  if (props.selected) {
    emit("deselect");
  } else {
    emit("select");
  }
};

const getPendingActionIcon = (actionName: string): IconNames => {
  const iconMap: Record<string, IconNames> = {
    Create: "plus",
    Update: "tilde",
    Refresh: "refresh",
    Destroy: "trash",
    Delete: "trash",
    Manual: "play",
  };
  return iconMap[actionName] || "play";
};

const getPendingActionIconClass = (actionName: string, hasFailed: boolean) => {
  // Red if failed, grey otherwise
  return hasFailed ? "text-destructive-500" : "text-neutral-500";
};

const getPendingActionTooltip = (
  actionName: string,
  count: number,
  hasFailed: boolean,
) => {
  const actionWord = actionName.toLowerCase();
  const plural = count > 1 ? "s" : "";

  if (hasFailed && count === 1) {
    return `1 pending ${actionWord} action failed`;
  } else if (hasFailed) {
    return `${count} pending ${actionWord} action${plural} (including failed)`;
  } else {
    return `${count} pending ${actionWord} action${plural}`;
  }
};

watch(
  () => [explore.focusedComponentIdx, gridTile],
  () => {
    if (
      gridTile.value &&
      gridTile.value.dataset.index ===
        explore.focusedComponentIdx.value?.toString()
    ) {
      explore.focusedComponentRef.value = gridTile.value;
    }
  },
  { immediate: true, deep: true },
);

const emit = defineEmits<{
  (e: "select"): void;
  (e: "deselect"): void;
}>();
</script>

<script lang="ts">
// Grid tiles need to have a fixed height - make sure this number matches its total height!
export const GRID_TILE_HEIGHT = 269;

export function getQualificationStatus(component: ComponentInList) {
  if (component.qualificationTotals.failed > 0) return "failure";
  if (component.qualificationTotals.warned > 0) return "warning";
  if (component.qualificationTotals.succeeded > 0) return "success";
  return "unknown";
}
</script>

<style scoped>
/* Running action animation - adapted from Lobby.vue */
@property --angle {
  syntax: "<angle>";
  inherits: false;
  initial-value: 0deg;
}

@keyframes borderRotate {
  100% {
    --angle: 360deg;
  }
}

.spinning-border {
  border: 1px solid;
  border-image: conic-gradient(
      from var(--angle),
      #06b6d4,
      #06b6d4 0.95turn,
      #0891b288 1turn
    )
    1;
  animation: borderRotate 3000ms linear infinite forwards;
  mask-image: radial-gradient(#000 0, #000 0);
}
</style>
