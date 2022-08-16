<template>
  <SiSearch />
  <ul class="overflow-y-auto">
    <li
      v-for="node in schematicNodes"
      :key="node.id"
      class="border-b-2 dark:border-neutral-600"
      @click="click(node)"
    >
      <span
        :class="
          selectedNode?.id === node.id
            ? ['bg-action-500 text-white']
            : ['hover:bg-action-400 hover:text-white']
        "
        :style="{
          'border-color': node.color, // Tailwind won't compile JS generated custom color styles, so we need to do this
        }"
        class="block px-2 py-2 border-l-8 group"
      >
        {{ node.name }}
        <i
          :class="
            selectedNode?.id === node.id
              ? ['bg-action-500 text-white']
              : ['text-neutral-500 group-hover:text-white']
          "
          class="text-sm pl-1"
        >
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
import {
  schematicData$,
  schematicSchemaVariants$,
} from "@/observable/schematic";
import {
  combineLatestWith,
  filter,
  firstValueFrom,
  map,
  switchMap,
} from "rxjs";
import { lastSelectedNode$, selectNode } from "@/observable/selection";
import { toast$ } from "@/observable/toast";
import { Node } from "@/organisms/SiCanvas/canvas/obj/node";

defineProps<{
  viewerEvent$: ViewerEventObservable["viewerEvent$"]; // Not used yet but necessary to link selected node between here and canvas
}>();

// NOTE(victor): This loads every schemaVariant color on a separate API call. This is very inefficient, but enough for user testing
// FIXME(victor): Refactor schematicData api to return component color
const schematicNodes = refFrom<Array<SchematicNode & { color: string }>>(
  schematicData$.pipe(
    combineLatestWith(schematicSchemaVariants$),
    filter(([_, variants]) => variants !== null),
    map(([sd]) => sd ?? { nodes: [], connections: [] }),
    switchMap(({ nodes }) =>
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

//Node Selection logic
const selectedNode = refFrom<Node | null>(lastSelectedNode$);

const click = async (node: SchematicNode & { color: string }) => {
  // TODO(victor): Remove dependency on deployment node
  const deploymentNodeId = await firstValueFrom(
    schematicData$.pipe(
      map((sd) => {
        if (!sd) {
          return null;
        }
        for (const node of sd.nodes) {
          if (node.kind.kind == "deployment") {
            return node.id;
          }
        }
        return null;
      }),
    ),
  );

  // NOTE(victor): This looks weird, but since the signup process always creates one deployment node like this, it works for now
  await selectNode(node.id, deploymentNodeId ?? -1);
  // Because there doesn't seem to be an easy way to convert from a SchematicNode to a PIXIjs compatible node object, we can't set the
  // lastSelectedNode$ object directly, which would make the attribute/code viewer work
  // TODO(victor): Make this call work after the canvas is reimplemented.
  // lastSelectedNode$.next(node);
  toast$.next({
    id: `component-selection-${node.id}`,
    success: false,
    title: `Error selecting node`,
    subtitle: "",
    message: "Please click canvas to select",
  });
};
</script>
