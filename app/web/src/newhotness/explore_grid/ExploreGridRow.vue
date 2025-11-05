<template>
  <div
    v-if="row.type === 'header'"
    :class="
      clsx(
        'flex flex-row items-center gap-xs px-xs',
        themeClasses('bg-neutral-200', 'bg-neutral-800'),
      )
    "
  >
    <CollapseExpandChevron
      :open="!row.collapsed"
      @click="emit('clickCollapse', row.title, !row.collapsed)"
    />
    <Icon
      v-if="titleIcon"
      :name="titleIcon.iconName"
      :tone="titleIcon.iconTone"
    />
    <span
      class="select-none"
      @click="emit('clickCollapse', row.title, !row.collapsed)"
    >
      {{ row.title }}
    </span>
    <PillCounter :count="row.count" class="text-xs" />
    <NewButton
      v-if="!row.collapsed"
      class="ml-auto"
      size="xs"
      :label="row.allSelected ? 'Deselect All' : 'Select All'"
      :disabled="row.count === 0"
      @click.stop.prevent="
        row.allSelected
          ? emit('deselectAllInSection', row.title)
          : emit('selectAllInSection', row.title)
      "
    />
  </div>
  <div
    v-else-if="row.type === 'defaultSubHeader'"
    :class="
      clsx(
        'flex flex-row items-center gap-sm px-xs',
        themeClasses('bg-neutral-200', 'bg-neutral-800'),
      )
    "
  >
    <CollapseExpandChevron
      :open="!row.collapsed"
      @click="emit('clickCollapse', row.subKey, !row.collapsed)"
    />
    <Icon :name="pickBrandIconByString(row.schemaCategory)" size="md" />
    <!-- TODO(Wendy) - we need a hover state here -->
    <div
      class="cursor-pointer select-none flex flex-row items-center gap-xs"
      @click="
        () =>
          row.type === 'defaultSubHeader' &&
          emit('componentNavigate', row.componentId)
      "
    >
      <span>{{ row.schemaName }}</span>
      <span :class="themeClasses('text-neutral-600', 'text-neutral-400')"
        >({{ row.componentName }})</span
      >

      <span
        v-if="defaultSubSourceValuesQuery.data.value"
        :class="
          clsx(
            'border rounded-sm p-2xs font-normal flex items-center',
            themeClasses(
              'border-neutral-400 bg-neutral-100 text-action-600',
              'border-neutral-500 bg-neutral-800 text-action-300',
            ),
          )
        "
        >{{ defaultSubSourceValuesQuery.data.value.prop }}</span
      >

      <span
        v-if="defaultSubSourceValuesQuery.data.value"
        :class="
          clsx(
            'border border-dashed rounded-sm font-normal flex flex-row items-center gap-2xs p-2xs',
            themeClasses(
              'border-neutral-400 bg-neutral-100',
              'border-neutral-500 bg-neutral-800',
            ),
          )
        "
      >
        <Icon name="arrow-outward" size="xs" />
        <TruncateWithTooltip>
          {{ defaultSubSourceValuesQuery.data.value.value }}
        </TruncateWithTooltip>
      </span>
    </div>
    <PillCounter :count="row.count" class="text-xs" />
    <NewButton
      v-if="!row.collapsed"
      class="ml-auto"
      size="xs"
      :label="row.allSelected ? 'Deselect All' : 'Select All'"
      :disabled="row.count === 0"
      @click.stop.prevent="
        row.allSelected
          ? emit('deselectAllInSection', row.subKey)
          : emit('selectAllInSection', row.subKey)
      "
    />
  </div>
  <div
    v-else-if="row.type === 'contentHeader'"
    :class="
      clsx(
        'flex flex-row items-center gap-xs px-xs',
        themeClasses('bg-neutral-200', 'bg-neutral-800'),
      )
    "
    @click="emit('clickCollapse', row.title, !row.collapsed)"
  >
    <CollapseExpandChevron :open="!row.collapsed" />
    <span class="select-none">
      {{ row.title }}
    </span>
    <!-- <Icon -->
    <!-- v-if="titleIcon" -->
    <!-- :name="titleIcon.iconName" -->
    <!-- :tone="titleIcon.iconTone" -->
    <!-- /> -->
    <!-- <PillCounter :count="row.count" class="text-xs" /> -->
  </div>
  <div v-else-if="row.type === 'pinnedContentRow'">
    <!-- We need an outer div because the card uses borders for the asset color. -->
    <div
      :class="
        clsx(
          'border rounded-sm cursor-pointer',
          pinnedBorderClasses(row.component.id),
        )
      "
    >
      <ComponentCard
        ref="exploreGridPinnedRef"
        :component="row.component"
        :data-index="row.dataIndex"
        @mouseenter="hover(row.component.id, true)"
        @mouseleave="hover(row.component.id, false)"
        @click.stop.left="
            (e: MouseEvent) =>
              row.type === 'pinnedContentRow' &&
              emit('childClicked', e, row.component.id, row.dataIndex)
        "
        @click.stop.right="
            (e: MouseEvent) =>
              row.type === 'pinnedContentRow' &&
              emit('childClicked', e, row.component.id, row.dataIndex)
        "
      >
        <template #endItems>
          <div
            v-tooltip="'Unpin'"
            :class="
              clsx(
                'bg-neutral-600 border border-neutral-600 rounded p-2xs',
                hoveredPinId === row.component.id &&
                  themeClasses(
                    'border-black bg-black',
                    'border-white bg-white',
                  ),
              )
            "
            @mouseenter="hoverPin(row.component.id, true)"
            @mouseleave="hoverPin(row.component.id, false)"
            @click.stop.left="emit('unpin', row.component.id)"
            @click.stop.right="emit('unpin', row.component.id)"
          >
            <Icon name="pin" size="sm" />
          </div>
        </template>
      </ComponentCard>
    </div>
  </div>
  <div
    v-else-if="row.type === 'contentRow'"
    :class="
      clsx(
        'flex flex-row items-start gap-sm',
        row.insideSection && 'px-xs',
        row.insideSection && themeClasses('bg-neutral-200', 'bg-neutral-800'),
      )
    "
  >
    <ExploreGridTile
      v-for="(component, columnIndex) in row.components"
      :key="component.id"
      :data-index="dataIndexForTileInRow(row, columnIndex)"
      :component="component"
      class="flex-1"
      :selected="isSelected(row, columnIndex)"
      :focused="exploreContext.focusedComponent.value?.id === component.id"
      :hovered="hoveredId === component.id"
      :hasFailedActions="
        exploreContext.componentsHaveActionsWithState.value.failed.has(
          component.id,
        )
      "
      :hasRunningActions="
        exploreContext.componentsHaveActionsWithState.value.running.has(
          component.id,
        )
      "
      :pendingActionCounts="
        exploreContext.componentsPendingActionNames.value.get(component.id)
      "
      @select="(e: MouseEvent) => emit('childSelect', dataIndexForTileInRow(row, columnIndex), e)"
      @deselect="(e: MouseEvent) => emit('childDeselect', dataIndexForTileInRow(row, columnIndex), e)"
      @mouseenter="hover(component.id, true)"
      @mouseleave="hover(component.id, false)"
      @click.stop.left="
        (e: MouseEvent) =>
          emit(
            'childClicked',
            e,
            component.id,
            dataIndexForTileInRow(row, columnIndex),
          )
      "
      @click.stop.right="
        (e: MouseEvent) =>
          emit(
            'childClicked',
            e,
            component.id,
            dataIndexForTileInRow(row, columnIndex),
          )
      "
    />
    <!--this fills in any extra spots in an unfilled row-->
    <div
      v-for="emptySpot in exploreContext.lanesCount.value -
      row.components.length"
      :key="emptySpot"
      class="flex-1"
    />
  </div>
  <div
    v-else-if="row.type === 'emptyRow'"
    :class="
      clsx(
        'flex items-center justify-center pb-xs',
        row.insideSection && 'px-xs',
        row.insideSection && themeClasses('bg-neutral-200', 'bg-neutral-800'),
      )
    "
  >
    <div
      :class="
        clsx(
          'flex flex-col items-center justify-center gap-md grow h-full',
          themeClasses(
            'bg-neutral-100 border border-neutral-400',
            'bg-neutral-800 border border-neutral-600',
          ),
        )
      "
    >
      <div
        :class="
          clsx(
            'p-sm rounded-full',
            themeClasses('bg-neutral-300', 'bg-neutral-700'),
          )
        "
      >
        <Icon name="check-circle-outline" />
      </div>
      <span>
        {{ emptyAreaData?.message ?? "Nothing to see here!" }}
      </span>
    </div>
  </div>
  <div
    v-else-if="row.type === 'filteredCounterRow'"
    class="flex items-center justify-center gap-1"
  >
    <span
      :class="
        clsx('text-xs', themeClasses('text-neutral-600', 'text-neutral-300'))
      "
    >
      {{ row.hiddenCount }} components hidden
    </span>
    <span
      :class="
        clsx('text-xs', themeClasses('text-neutral-600', 'text-neutral-300'))
      "
    >
      |
    </span>
    <button
      :class="
        clsx(
          'text-xs underline cursor-pointer hover:no-underline',
          themeClasses(
            'text-neutral-600 hover:text-black',
            'text-neutral-300 hover:text-white',
          ),
        )
      "
      @click="emit('resetFilter')"
    >
      Reset Filter
    </button>
  </div>
  <!-- This is subtle, but important. We need a div here, even if empty. -->
  <div v-else>
    <!-- footer area -->
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { computed, inject, ref } from "vue";
import * as _ from "lodash-es";
import {
  themeClasses,
  Icon,
  PillCounter,
  IconNames,
  Tones,
  TruncateWithTooltip,
  NewButton,
  CollapseExpandChevron,
} from "@si/vue-lib/design-system";
import { tw } from "@si/vue-lib";
import { useQuery } from "@tanstack/vue-query";
import {
  AttributeTree,
  ComponentInList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import { ComponentId } from "@/api/sdf/dal/component";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import ComponentCard from "../ComponentCard.vue";
import ExploreGridTile from "./ExploreGridTile.vue";
import { assertIsDefined, ExploreContext } from "../types";
import { pickBrandIconByString } from "../util";
import { useContext } from "../logic_composables/context";

const props = defineProps<{
  row: ExploreGridRowData;
}>();

const exploreContext = inject<ExploreContext>("EXPLORE_CONTEXT");
assertIsDefined<ExploreContext>(exploreContext);

const ctx = useContext();

const key = useMakeKey();
const args = useMakeArgs();

const enableDefaultSubSourceValuesQuery = computed(
  () => ctx.queriesEnabled && props.row.type === "defaultSubHeader",
);
const defaultSubSourceValuesQuery = useQuery<
  { prop: string; value: string } | undefined
>({
  enabled: enableDefaultSubSourceValuesQuery,
  queryKey: key(
    EntityKind.AttributeTree,
    props.row.type === "defaultSubHeader" ? props.row.componentId : "",
    "DefaultSubHeader",
  ),
  queryFn: async () => {
    // This is just a type assertion, the query should only be enabled if this
    // is already true, so we'll never hit this return
    if (props.row.type !== "defaultSubHeader") {
      return;
    }
    const attributeTree = await bifrost<AttributeTree>(
      args(EntityKind.AttributeTree, props.row.componentId),
    );
    const value =
      attributeTree &&
      Object.values(attributeTree.attributeValues).find(
        (av) =>
          props.row.type === "defaultSubHeader" && av.path === props.row.path,
      );

    const prop = value?.propId
      ? attributeTree?.props[value.propId]?.name
      : undefined;
    if (!prop) {
      return;
    }
    const sourceValue = value?.secret
      ? value.secret.name
      : value?.value?.toString();
    if (!sourceValue) {
      return;
    }

    return {
      prop,
      value: sourceValue,
    };
  },
});

interface TitleIcon {
  iconName: IconNames;
  iconTone: Tones;
}

const isSelected = (row: ExploreGridRowData, columnIndex: number) =>
  exploreContext.selectedComponentIndexes.has(
    dataIndexForTileInRow(row, columnIndex),
  );

const titleIcon = computed((): TitleIcon | null => {
  if (props.row.type !== "header") return null;

  switch (props.row.title) {
    case "Failed qualifications":
      return {
        iconName: "x-hex-outline",
        iconTone: "destructive",
      };
    case "Warnings":
      return {
        iconName: "alert-triangle-outline",
        iconTone: "warning",
      };
    case "Passed qualifications":
      return {
        iconName: "check-hex-outline",
        iconTone: "success",
      };
    case "Unknown qualification status":
      return {
        iconName: "question-circle",
        iconTone: "warning",
      };
    default:
      return null;
  }
});

interface EmptyAreaData {
  message: string;
}

const emptyAreaData = computed((): EmptyAreaData | null => {
  if (props.row.type !== "emptyRow") return null;

  switch (props.row.groupName) {
    case "Failed qualifications":
      return {
        message: "No failed qualifications",
      };
    case "Unknown qualification status":
      return {
        message: "No components with unknown qualifications",
      };
    case "Warnings":
      return {
        message: "No warnings on qualifications",
      };
    case "Passed qualifications":
      return {
        message: "No passing qualifications",
      };
    case "With Diffs":
      return {
        message: "No components have been changed so far",
      };
    case "Without Diffs":
      return {
        message: "All components have been changed!",
      };
    case "Upgradable":
      return {
        message: "No components ready for an upgrade so far",
      };
    case "Up to date":
      return {
        message: "All components are upgradable right now",
      };
    case "Default Subscription Users":
      return {
        message: "No components are using this default subscription",
      };
    default:
      return null;
  }
});

// You can only have one card in a row, but you have can multiple tiles in a row.
const exploreGridPinnedRef = ref<InstanceType<typeof ComponentCard>>();

const dataIndexForTileInRow = (row: ExploreGridRowData, idx: number) => {
  if (row.type !== "contentRow") return -1;

  return row.chunkInitialId + idx;
};

const hoveredId = ref<ComponentId | undefined>(undefined);
const hover = (componentId: ComponentId, hovered: boolean) => {
  if (hovered) {
    hoveredId.value = componentId;
    emit("childHover", componentId);
  } else {
    hoveredId.value = undefined;
    emit("childUnhover", componentId);
  }
};

// NOTE(nick): at the time of writing, you could only pin one component at a time. However, the
// hover detection was written to handle multiple pinned components. That is why it works with a
// "componentId" rather than "boolean" for the pin hovering logic.
const hoveredPinId = ref<ComponentId | undefined>(undefined);
const hoverPin = (pinnedComponentId: ComponentId, hovered: boolean) => {
  if (hovered) {
    hoveredPinId.value = pinnedComponentId;
  } else if (hoveredPinId.value === pinnedComponentId) {
    hoveredPinId.value = undefined;
  }

  // Invert the hover state for the entire component. If you are hovering the pin, you are not
  // hovering the component and vice versa.
  hover(pinnedComponentId, !hovered);
};

// Provide a background color because we need to fill the empty space between the div's border and
// the inner border for the asset color.
const pinnedBorderClasses = (componentId: string) => {
  const focused = exploreContext.focusedComponent.value?.id === componentId;
  if (focused)
    return themeClasses(
      tw`border-action-500 bg-action-500`,
      tw`border-action-300 bg-action-300`,
    );
  else if (hoveredId.value === componentId)
    return themeClasses(tw`border-black bg-black`, tw`border-white bg-white`);
  else return "border-neutral-600 bg-neutral-600";
};

const emit = defineEmits<{
  (e: "unpin", componentId: ComponentId): void;
  (e: "childHover", componentId: ComponentId): void;
  (e: "childUnhover", componentId: ComponentId): void;
  (
    e: "childClicked",
    event: MouseEvent,
    componentId: ComponentId,
    componentIdx: number,
  ): void;
  (e: "clickCollapse", title: string, collapsed: boolean): void;
  (e: "childSelect", componentIdx: number, event?: MouseEvent): void;
  (e: "childDeselect", componentIdx: number, event?: MouseEvent): void;
  (e: "componentNavigate", componentId: ComponentId): void;
  (e: "resetFilter"): void;
  (e: "selectAllInSection", sectionKey: string): void;
  (e: "deselectAllInSection", sectionKey: string): void;
}>();
</script>

<script lang="ts">
export type ExploreGridRowData =
  | {
      type: "contentRow";
      components: ComponentInList[];
      chunkInitialId: number;
      insideSection: boolean;
    }
  | {
      type: "pinnedContentRow";
      component: ComponentInList;
      dataIndex: number;
    }
  | {
      type: "header";
      title: string;
      count: number;
      collapsed: boolean;
      allSelected: boolean;
    }
  | {
      type: "footer";
    }
  | {
      type: "emptyRow";
      groupName: string;
      insideSection: boolean;
    }
  | {
      type: "filteredCounterRow";
      hiddenCount: number;
    }
  | {
      type: "contentHeader";
      title: string;
      collapsed: boolean;
    }
  | {
      type: "defaultSubHeader";
      schemaName: string;
      schemaCategory: string;
      componentName: string;
      componentId: ComponentId;
      path: string;
      collapsed: boolean;
      count: number;
      subKey: string; // Needed for collapse handling
      allSelected: boolean;
    };
</script>

<style lang="css" scoped>
section.grid.explore {
  grid-template-columns: minmax(0, 70%) minmax(0, 30%);
  grid-template-rows: 100%;
  grid-template-areas: "main right";
}

section.grid.map {
  grid-template-columns: 100%;
  grid-template-rows: 100%;
  grid-template-areas: "main";
}

div.main {
  grid-area: "main";
}

div.right {
  grid-area: "right";
}
</style>
