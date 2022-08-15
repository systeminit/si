<template>
  <SiSearch />

  <ul class="overflow-y-auto">
    <li
      v-for="node in schematicNodes"
      :key="node.id"
      class="border-b-2 dark:border-neutral-600"
    >
      <span
        :style="{
          'border-color': node.color, // Tailwind won't compile JS generated custom color styles, so we need to do this
        }"
        class="block px-1 py-0.5 border-l-8"
      >
        {{ node.name }}
        <i class="text-neutral-500 text-sm">
          {{ node.title }}
        </i>
      </span>
    </li>
  </ul>
</template>

<script lang="ts" setup>
import SiSearch from "@/molecules/SiSearch.vue";
import { ViewerEventObservable } from "@/organisms/SiCanvas/viewer_event";
import { SchematicNode, variantById } from "@/api/sdf/dal/schematic";
import { refFrom } from "vuse-rx";
import { schematicData$ } from "@/observable/schematic";
import { concatMap, map } from "rxjs";

defineProps<{
  viewerEvent$: ViewerEventObservable["viewerEvent$"]; // Not used yet but necessary to link selected node between here and canvas
}>();

// NOTE(victor): This loads every schemaVariant color on a separate API call. This is very inefficient, but enough for user testing
// FIXME(victor): Refactor schematicData api to return component color
const schematicNodes = refFrom<Array<SchematicNode & { color: string }>>(
  schematicData$.pipe(
    map((sd) => sd ?? { nodes: [], connections: [] }),
    concatMap(({ nodes }) =>
      Promise.all(
        nodes
          .filter((n) => n.kind.kind !== "deployment")
          .map(async (n) => {
            const { color } = await variantById(n.schemaVariantId);

            return {
              ...n,
              color: "#" + color.toString(16).padStart(6, "0"),
            };
          }),
      ),
    ),
  ),
);
</script>
