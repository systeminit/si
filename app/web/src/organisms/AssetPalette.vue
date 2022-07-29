<template>
  <SiSearch />

  <p
    class="border-b-2 dark:border-neutral-600 text-xs font-light leading-tight px-3 py-1"
  >
    Get Started by dragging the assets that you wish to include in your
    application into the canvas to the right
  </p>

  <ul class="overflow-y-auto">
    <SiCollapsible
      v-for="(category, category_index) in assetCategories"
      :key="category_index"
      :label="category.name"
      as="li"
      content-as="ul"
    >
      <li v-for="(node, node_index) in category.assets" :key="node_index">
        <SiNodeSprite
          :class="activeNode === node.id ? 'bg-action-500' : ''"
          :color="node.color"
          :name="node.name"
          class="border-b-2 dark:border-neutral-600 hover:bg-action-500 dark:text-white hover:text-white hover:cursor-pointer"
          @click="setActiveNode(node, $event)"
        />
      </li>
    </SiCollapsible>
  </ul>
</template>

<script lang="ts" setup>
import SiNodeSprite from "@/molecules/SiNodeSprite.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import SiSearch from "@/molecules/SiSearch.vue";
import { ref } from "vue";
import { combineLatestWith, firstValueFrom } from "rxjs";
import { SchematicService } from "@/service/schematic";
import {
  SchematicKind,
  SchematicNode,
  SchematicNodeTemplate,
  variantById,
} from "@/api/sdf/dal/schematic";
import {
  NodeAddEvent,
  ViewerEventObservable,
} from "@/organisms/SiCanvas/viewer_event";
import { ApplicationService } from "@/service/application";
import { schematicSchemaVariants$ } from "@/observable/schematic";
import { Category, Item } from "@/api/sdf/dal/menu";
import { utils as PixiUtils } from "pixi.js";
import { untilUnmounted } from "vuse-rx";

const props = defineProps<{
  viewerEvent$: ViewerEventObservable["viewerEvent$"];
}>();

// TODO(victor): this types shouldn't be here, but since we probably need to refactor the way nodes/assets are organized in the backend I'll keep them for now
interface Asset {
  id: number;
  name: string;
  template: SchematicNodeTemplate;
  color: string;
}

interface AssetCategory {
  name: string;
  assets: Asset[];
}

const assetCategories = ref<AssetCategory[]>([]);

// FIXME(nick,victor): temporary measure to populate the assetCategories dynamically based on the application.
ApplicationService.currentApplication()
  .pipe(combineLatestWith(schematicSchemaVariants$))
  .pipe(untilUnmounted)
  .subscribe(async ([application, schemaVariants]) => {
    if (application === null || schemaVariants === null) {
      assetCategories.value = [];
      return;
    }

    const nodeAddMenu = await firstValueFrom(
      SchematicService.getNodeAddMenu({
        menuFilter: {
          rootComponentId: application.id,
          schematicKind: SchematicKind.Component,
        },
      }),
    );

    if (nodeAddMenu.error) {
      assetCategories.value = [];
      return;
    }

    // TODO(victor): when the old interface goes, the API probably could return the expected structure and we won't need this conversion
    // for now, we assume the endpoint returns an array of `api/sdf/dal/menu.Category` containing an array of `api/sdf/dal/menu.Item`
    assetCategories.value = await Promise.all(
      nodeAddMenu
        .filter((c) => c.kind === "category")
        .map(async (c) => {
          const { name, items } = c as Category;

          const assets = [];

          for (const item of items) {
            if (item.kind !== "item") continue;

            const { name, schema_id: schemaId } = item as Item;

            const template = await firstValueFrom(
              SchematicService.getNodeTemplate({ schemaId }),
            );

            if (template.error) continue;

            const { color } = await variantById(template.schemaVariantId);

            assets.push({
              name: name,
              id: schemaId,
              template,
              color: PixiUtils.hex2string(color),
            });
          }

          return {
            name,
            assets,
          };
        }),
    );
  });

const activeNode = ref<number | undefined>();

const setActiveNode = (e: Asset, _event: MouseEvent) => {
  // TODO(victor): This code makes it so that clicking the selected node deselects it. That should probably change when node addiction is handled by an observable
  activeNode.value = e.id !== activeNode.value ? e.id : undefined;

  // TODO(nick): temporarily embedding the add node into the active node event.
  addNode(e.id, e.template, _event);
};

const addNode = async (
  schemaId: number,
  schemaTemplate: SchematicNodeTemplate,
  _event: MouseEvent,
) => {
  // Generates fake node from template
  const node: SchematicNode = {
    id: -1,
    kind: { kind: schemaTemplate.kind, componentId: -1 },
    title: schemaTemplate.title,
    name: schemaTemplate.name,
    positions: [
      {
        schematicKind:
          schemaTemplate.kind === "component"
            ? SchematicKind.Component
            : SchematicKind.Deployment,
        x: 350,
        y: 0,
      },
    ],
    schemaVariantId: schemaTemplate.schemaVariantId,
  };

  const event = new NodeAddEvent({
    node,
    schemaId,
  });

  props.viewerEvent$.next(event);
  // TODO(victor) we should subscribe to viewerEvents to sync the selected node with the real state
};
</script>
