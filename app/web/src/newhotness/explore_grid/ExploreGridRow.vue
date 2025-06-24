<template>
  <div
    v-if="row.type === 'header'"
    class="flex flex-row items-center bg-neutral-900 px-xs gap-xs"
  >
    <Icon
      v-if="titleIcon"
      :name="titleIcon.iconName"
      :tone="titleIcon.iconTone"
    />
    <span>
      {{ row.title }}
    </span>
    <PillCounter :count="row.count" class="text-xs" />
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
      :data-index="computeIdx(row, columnIndex)"
      :component="component"
      class="flex-1"
      :class="clsx(tileClasses(component.id))"
      @mouseenter="hover(component.id, true)"
      @mouseleave="hover(component.id, false)"
      @click.stop.left="
        (e) =>
          emit('childClicked', e, component.id, computeIdx(row, columnIndex))
      "
      @click.stop.right="
        (e) =>
          emit('childClicked', e, component.id, computeIdx(row, columnIndex))
      "
    />
    <!--this fills in any extra spots in an unfilled row-->
    <div
      v-for="emptySpot in lanesCount - row.components.length"
      :key="emptySpot"
      class="flex-1"
    />
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
import ExploreGridTile from "./ExploreGridTile.vue";

const props = defineProps<{
  row: ExploreGridRowData;
  lanesCount: number;
  focusedComponentId?: ComponentId;
}>();

const emit = defineEmits<{
  (e: "childHover", componentId: ComponentId): void;
  (e: "childUnhover", componentId: ComponentId): void;
  (
    e: "childClicked",
    event: MouseEvent,
    componentId: ComponentId,
    componentIdx: number,
  ): void;
}>();

interface TitleIcon {
  iconName: IconNames;
  iconTone: Tones;
}

const titleIcon = computed((): TitleIcon | null => {
  if (props.row.type !== "header") return null;

  switch (props.row.title) {
    case "Passed":
      return {
        iconName: "check-hex-outline",
        iconTone: "success",
      };
    case "Failed":
      return {
        iconName: "x-hex-outline",
        iconTone: "destructive",
      };
    default:
      return null;
  }
});

const exploreGridTileRefs = ref<InstanceType<typeof ExploreGridTile>[]>();

const computeIdx = (row: ExploreGridRowData, idx: number) => {
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

const tileClasses = (componentId: string) => {
  const focused = props.focusedComponentId === componentId;
  if (focused)
    return themeClasses(tw`border-action-500`, tw`border-action-300`);
  else if (hoveredId.value === componentId)
    return themeClasses(tw`border-black`, tw`border-white`);
  else return "";
};

defineExpose({ exploreGridTileRefs });
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
      type: "header";
      title: string;
      count: number;
    }
  | {
      type: "footer";
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
