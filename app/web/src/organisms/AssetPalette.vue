<template>
  <SiSearch />

  <p
    class="border-b-2 dark:border-[#525252] text-xs font-light leading-tight px-3 py-1"
  >
    Get Started by dragging the assets that you wish to include in your
    application into the canvas to the right
  </p>

  <ul class="overflow-y-auto">
    <SiCollapsible
      v-for="(category, category_index) in assetCategories"
      :key="category_index"
      as="li"
      content-as="ul"
      :label="category.name"
    >
      <li v-for="(node, node_index) in category.assets" :key="node_index">
        <SiNodeSprite
          :color="category.color"
          :name="node.name"
          class="border-b-2 dark:border-[#525252] hover:bg-[#2F80ED] dark:text-white hover:text-white hover:cursor-pointer"
          :class="activeNode === node.id ? 'bg-[#2F80ED]' : ''"
          @click="setActiveNode(node, $event)"
        />
      </li>
    </SiCollapsible>
  </ul>
</template>

<script setup lang="ts">
import SiNodeSprite from "@/molecules/SiNodeSprite.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiSearch from "@/molecules/SiSearch.vue";
import { ref } from "vue";
import { firstValueFrom } from "rxjs";
import { SchematicService } from "@/service/schematic";
import { GlobalErrorService } from "@/service/global_error";
import { SchematicKind, SchematicNode } from "@/api/sdf/dal/schematic";
import {
  NodeAddEvent,
  ViewerEventObservable,
} from "@/organisms/SchematicViewer/viewer_event";

const props = defineProps<{
  viewerEvent$: ViewerEventObservable["viewerEvent$"];
}>();

interface Asset {
  id: number;
  name: string;
}

interface AssetCategory {
  name: string;
  color: string;
  assets: Asset[];
}

const assetCategories: AssetCategory[] = [
  {
    name: "Kubernetes",
    color: "#00F",
    assets: [{ id: 1, name: "Service (placeholder)" }],
  },
  {
    name: "Cloud",
    color: "#F00",
    assets: [
      { id: 2, name: "Secret" },
      { id: 3, name: "Database" },
    ],
  },
  {
    name: "Something with a really really long name oh no oh no",
    color: "#0FF",
    assets: [
      {
        id: 4,
        name: "Oh no this thing with a really really long name has an asset with a really long name inside",
      },
      {
        id: 5,
        name: "Oh no this thing with a really really long name has an asset with a really really long name inside",
      },
      { id: 6, name: "Help me I'm stuck in a cloud component factory" },
    ],
  },
];

// NodeSprite
// NodeSprite normal hover focus disabled
// NodeCategory

const activeNode = ref<number | undefined>();

const setActiveNode = (e: Asset, _event: MouseEvent) => {
  // TODO(victor): This code makes it so that clicking the selected node deselects it. That should probably change when node addiction is handled by an observable
  activeNode.value = e.id !== activeNode.value ? e.id : undefined;

  // TODO(nick): temporarily embedding the add node into the active node event.
  addNode(3, _event);
};

const addNode = async (schemaId: number, _event: MouseEvent) => {
  const template = await firstValueFrom(
    SchematicService.getNodeTemplate({ schemaId }),
  );
  if (template.error) {
    GlobalErrorService.set(template);
    return;
  }

  // Generates fake node from template
  const node: SchematicNode = {
    id: -1,
    kind: { kind: template.kind, componentId: -1 },
    title: template.title,
    name: template.name,
    positions: [
      {
        schematicKind:
          template.kind === "component"
            ? SchematicKind.Component
            : SchematicKind.Deployment,
        x: 350,
        y: 0,
      },
    ],
    schemaVariantId: template.schemaVariantId,
  };

  const event = new NodeAddEvent({ node, schemaId: schemaId });

  props.viewerEvent$.next(event);
};
</script>
