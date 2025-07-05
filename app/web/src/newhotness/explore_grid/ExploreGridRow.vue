<template>
  <div
    v-if="row.type === 'header'"
    class="flex flex-row items-center bg-neutral-900 px-xs gap-xs"
    @click="emit('clickCollapse', row.title, !row.collapsed)"
  >
    <Icon :name="row.collapsed ? 'chevron--right' : 'chevron--down'" />
    <Icon
      v-if="titleIcon"
      :name="titleIcon.iconName"
      :tone="titleIcon.iconTone"
    />
    <span class="select-none">
      {{ row.title }}
    </span>
    <PillCounter :count="row.count" class="text-xs" />
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
        @hoverPin="
          (hovered) =>
            row.type === 'pinnedContentRow' && hover(row.component.id, !hovered)
        "
        @mouseenter="hover(row.component.id, true)"
        @mouseleave="hover(row.component.id, false)"
        @unpin="emit('unpin', row.component.id)"
        @click.stop.left="
          (e) =>
            row.type === 'pinnedContentRow' &&
            emit('childClicked', e, row.component.id, row.dataIndex)
        "
        @click.stop.right="
          (e) =>
            row.type === 'pinnedContentRow' &&
            emit('childClicked', e, row.component.id, row.dataIndex)
        "
      >
        <template #endItems>
          <div
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
        row.insideSection && 'bg-neutral-900 px-xs',
      )
    "
  >
    <ExploreGridTile
      v-for="(component, columnIndex) in row.components"
      ref="exploreGridTileRefs"
      :key="component.id"
      :data-index="dataIndexForTileInRow(row, columnIndex)"
      :component="component"
      class="flex-1"
      showSelectionCheckbox
      :selected="
        selectedComponentIndexes.has(dataIndexForTileInRow(row, columnIndex))
      "
      :focused="focusedComponentId === component.id"
      :hovered="hoveredId === component.id"
      :hasFailedActions="componentsWithFailedActions.has(component.id)"
      @select="emit('childSelect', dataIndexForTileInRow(row, columnIndex))"
      @deselect="emit('childDeselect', dataIndexForTileInRow(row, columnIndex))"
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
      v-for="emptySpot in lanesCount - row.components.length"
      :key="emptySpot"
      class="flex-1"
    />
  </div>
  <div
    v-else-if="row.type === 'emptyRow'"
    class="flex items-center justify-center bg-neutral-900 pb-xs px-xs"
  >
    <div
      class="flex flex-col items-center justify-center gap-md bg-neutral-800 border border-neutral-600 grow h-full"
    >
      <div class="bg-neutral-700 p-sm rounded-full">
        <Icon name="check-circle-outline" />
      </div>
      <span>
        {{ emptyAreaData?.message ?? "Nothing to see here!" }}
      </span>
    </div>
  </div>
  <!-- This is subtle, but important. We need a div here, even if empty. -->
  <div v-else>
    <!-- footer area -->
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { computed, ref } from "vue";
import * as _ from "lodash-es";
import {
  themeClasses,
  Icon,
  PillCounter,
  IconNames,
  Tones,
} from "@si/vue-lib/design-system";
import { tw } from "@si/vue-lib";
import { ComponentInList } from "@/workers/types/entity_kind_types";
import { ComponentId } from "@/api/sdf/dal/component";
import ComponentCard from "../ComponentCard.vue";
import ExploreGridTile from "./ExploreGridTile.vue";

const props = defineProps<{
  row: ExploreGridRowData;
  lanesCount: number;
  selectedComponentIndexes: Set<number>;
  focusedComponentId?: ComponentId;
  componentsWithFailedActions: Set<ComponentId>;
}>();

interface TitleIcon {
  iconName: IconNames;
  iconTone: Tones;
}

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
    default:
      return null;
  }
});

// You can only have one card in a row, but you have can multiple tiles in a row.
const exploreGridPinnedRef = ref<InstanceType<typeof ComponentCard>>();
const exploreGridTileRefs = ref<InstanceType<typeof ExploreGridTile>[]>();
const exploreGridComponentRefs = computed(() =>
  _.compact([exploreGridPinnedRef.value, ...(exploreGridTileRefs.value ?? [])]),
);

const dataIndexForTileInRow = (row: ExploreGridRowData, idx: number) => {
  if (row.type !== "contentRow") return -1;

  return row.chunkInitialId + idx;
};

const hoveredId = ref<ComponentId | undefined>(undefined);
const hover = (componentId: ComponentId, hovered: boolean) => {
  if (hovered) {
    hoveredId.value = componentId;
    emit("childUnhover", componentId);
  } else {
    hoveredId.value = undefined;
    emit("childHover", componentId);
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
  const focused = props.focusedComponentId === componentId;
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
  (e: "childSelect", componentIdx: number): void;
  (e: "childDeselect", componentIdx: number): void;
}>();

defineExpose({ exploreGridComponentRefs });
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
    }
  | {
      type: "footer";
    }
  | {
      type: "emptyRow";
      groupName: string;
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
