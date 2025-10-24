<template>
  <div
    ref="gridTile"
    :class="
      clsx(
        'component tile',
        'cursor-pointer rounded-sm overflow-hidden relative select-none border',
        hasRunningActions && 'spinning-border',
        !hasRunningActions && [
          selected || focused
            ? [
                themeClasses('border-neutral-600', 'border-neutral-400'),
                themeClasses(
                  'outline outline-1 outline-neutral-600',
                  'outline outline-1 outline-neutral-400',
                ),
              ]
            : hasFailedActions
            ? themeClasses('border-destructive-500', 'border-destructive-400')
            : [
                isHovering
                  ? themeClasses('border-action-700', 'border-action-600')
                  : themeClasses('border-neutral-400', 'border-neutral-600'),
              ],
        ],
        themeClasses('bg-shade-0', 'bg-neutral-900'),
        component.toDelete && 'opacity-70',
      )
    "
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <header
      :class="
        clsx(
          'p-xs pr-md',
          !component.toDelete && [
            isHovering
              ? themeClasses('bg-neutral-300', 'bg-neutral-600')
              : themeClasses('bg-neutral-200', 'bg-neutral-800'),
          ],
          noIconShowing && '!grid-cols-[20px_minmax(0,_1fr)]',
        )
      "
      :style="
        noIconShowing ? `grid-template-areas: 'logo h2' 'logo h3';` : undefined
      "
    >
      <Icon
        :name="getAssetIcon(component.schemaCategory)"
        size="sm"
        class="place-self-center my-auto mt-[7px]"
      />
      <h2>
        <TruncateWithTooltip
          class="pb-xs text-sm"
          :reverse="featureFlagsStore.REVERSE_TRUNCATION"
        >
          {{ component.name }}
        </TruncateWithTooltip>
      </h2>
      <h3>
        <TruncateWithTooltip
          class="pb-xs text-xs"
          :reverse="featureFlagsStore.REVERSE_TRUNCATION"
        >
          {{ component.schemaName }}
        </TruncateWithTooltip>
      </h3>
      <Icon
        v-if="component.toDelete"
        v-tooltip="'This component is set for deletion'"
        name="trash"
        size="sm"
        :class="
          clsx(
            themeClasses('text-destructive-600', 'text-destructive-300'),
            'mt-[7px]',
          )
        "
      />
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
        v-tooltip="'This component can be upgraded'"
        name="bolt-outline"
        size="sm"
        :class="
          clsx(themeClasses('text-success-500', 'text-success-400'), 'mt-[7px]')
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
          <Icon
            name="tilde-circle"
            :class="themeClasses('text-warning-500', 'text-warning-300')"
            size="sm"
          />
          <div class="text-sm">Diff</div>
          <TextPill
            v-if="component.diffStatus === 'Added'"
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
            v-else-if="component.diffStatus === 'Modified'"
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
          <ActionPills :actionCounts="pendingActionCounts" mode="grid" />
        </template>
      </li>
      <!-- NOTE: when coming from the Map page we don't have accurate outputCount, hiding this -->
      <template v-if="!props.hideConnections">
        <hr :class="themeClasses('border-neutral-400', 'border-neutral-600')" />
        <li>
          <Icon name="input-connection" size="sm" />
          <div class="text-sm">Incoming</div>
          <PillCounter
            :count="component.inputCount"
            size="sm"
            class="ml-auto"
          />
        </li>
        <li>
          <Icon name="output-connection" size="sm" />
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
            : [
                isHovering
                  ? themeClasses('border-black', 'border-white')
                  : themeClasses('border-neutral-400', 'border-neutral-600'),
              ],
          selected
            ? themeClasses('bg-black', 'bg-neutral-600')
            : isHovering
            ? themeClasses('bg-neutral-300', 'bg-neutral-600')
            : themeClasses('bg-neutral-200', 'bg-neutral-800'),
        )
      "
      @click.stop.prevent.left="toggleSelection($event)"
      @click.stop.prevent.right="toggleSelection($event)"
    >
      <Icon
        v-if="selected"
        name="check"
        size="xs"
        class="absolute top-[-1px] left-[-1px] text-white"
      />
    </div>
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
import { computed, inject, ref, watch } from "vue";
import { ComponentInList } from "@/workers/types/entity_kind_types";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { getAssetIcon } from "../util";
import { assertIsDefined, Context, ExploreContext } from "../types";
import ComponentTileQualificationStatus from "../ComponentTileQualificationStatus.vue";
import ActionPills from "../ActionPills.vue";

const props = defineProps<{
  component: ComponentInList;
  selected?: boolean;
  focused?: boolean;
  hovered?: boolean;
  hideConnections?: boolean;
  hasFailedActions?: boolean;
  hasRunningActions?: boolean;
  pendingActionCounts?: Record<string, { count: number; hasFailed: boolean }>;
}>();

const featureFlagsStore = useFeatureFlagsStore();

const isHovering = ref(false);
const showSelectionCheckbox = computed(
  () => isHovering.value || props.selected,
);

const handleMouseEnter = (event: MouseEvent) => {
  isHovering.value = true;
  emit("mouseenter", event);
};

const handleMouseLeave = (event: MouseEvent) => {
  isHovering.value = false;
  emit("mouseleave", event);
};

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
    props.component.diffStatus && props.component.diffStatus !== "None";

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
const toggleSelection = (event: MouseEvent) => {
  if (props.selected) {
    emit("deselect", event);
  } else {
    emit("select", event);
  }
};

const noIconShowing = computed(
  () =>
    !props.component.toDelete &&
    !props.component.hasSocketConnections &&
    !canBeUpgraded.value,
);

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
  (e: "select", event: MouseEvent): void;
  (e: "deselect", event: MouseEvent): void;
  (e: "mouseenter", event: MouseEvent): void;
  (e: "mouseleave", event: MouseEvent): void;
}>();
</script>

<script lang="ts">
// Grid tiles need to have a fixed height - make sure this number matches its total height!
export const GRID_TILE_HEIGHT = 269;
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
  outline: 1px solid #06b6d4;
  outline-offset: 0px;
}
</style>
