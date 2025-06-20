<template>
  <div
    v-if="row.type === 'header'">
    {{row}}
  </div>
  <div
    class="flex flex-row items-center gap-sm my-2"
    v-else
  >
      <ComponentGridTile
        v-for="(component, columnIndex) in row.components"
        ref="componentGridTileRefs"
        :key="component.id"
        :data-index="row.chunkIndex * lanesCount + columnIndex"
        :component="component"
        class="flex-1"
        :class="clsx(tileClasses(component.id))"
        @mouseenter="$emit('childHover', component.id)"
        @mouseleave="$emit('childUnhover', component.id)"
        @click.stop.left="(e) => $emit('childLeftClick', e, component.id)"
        @click.stop.right="(e) => $emit('childRightClick', e, component.id)"
      />
        <!--this fills in any extra spots in the last row-->
        <div
          v-for="emptySpot in lanesCount - row.components.length"
          :key="emptySpot"
          class="flex-1"
        />

  </div>

</template>

<script lang="ts" setup>
import clsx from "clsx";
import ComponentGridTile, { GRID_TILE_HEIGHT } from "../ComponentGridTile.vue";
import { computed, ref, reactive } from "vue";
import * as _ from "lodash-es";
import { ComponentInList } from "@/workers/types/entity_kind_types";
import {
  KeyDetails,
  keyEmitter,
  windowResizeEmitter,
  windowWidthReactive,
} from "../logic_composables/emitters";
import {
  themeClasses,
  VormInput,
  VButton,
  DropdownMenuButton,
  DropdownMenuItem,
  Icon,
} from "@si/vue-lib/design-system";
import ComponentContextMenu from "../ComponentContextMenu.vue";
import { tw } from "@si/vue-lib";
import { useRouter, useRoute } from "vue-router";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { ComponentId } from "@/api/sdf/dal/component";

const props = defineProps<{
  row: ExploreGridRowData;
  lanesCount: number;
  focusedId: string;
  hoverId: string;
}>();

const tileClasses = (componentId: string) => {
  const hovered = props.hoverId === componentId;
  const focused = props.focusedId === componentId;
  if (focused)
    return themeClasses(tw`border-action-500`, tw`border-action-300`);
  else if (hovered) return themeClasses(tw`border-black`, tw`border-white`);
  else return "";
};


defineEmits<{
  (e: "childHover", componentIdx: string): void;
  (e: "childUnhover", componentIdx: string): void;
  (e: "childLeftClick", event: MouseEvent, componentIdx: string): void;
  (e: "childRightClick", event :MouseEvent, componentIdx: string): void;

}>();

</script>

<script lang="ts">
export type ExploreGridRowData = {
  type: "contentRow",
  components: ComponentInList[],
  chunkIndex: number
} | {
  type: "header",
  title: string,
  count: number,
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
