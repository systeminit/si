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
      as="li"
      content-as="ul"
      :label="category.name"
    >
      <li v-for="(node, node_index) in category.assets" :key="node_index">
        <SiNodeSprite
          :color="category.color"
          :name="node.name"
          class="border-b-2 dark:border-neutral-600 hover:bg-action-500 dark:text-white hover:text-white hover:cursor-pointer"
          :class="activeNode === node.id ? 'bg-action-500' : ''"
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
} from "@/organisms/SiCanvas/viewer_event";
import { Category, Item, MenuItem } from "@/api/sdf/dal/menu";
import { ApiResponse } from "@/api/sdf";
import { ApplicationService } from "@/service/application";

const props = defineProps<{
  viewerEvent$: ViewerEventObservable["viewerEvent$"];
}>();

// TODO(victor): this types shouldn't be here, but since we probably need to refactor the way nodes/assets are organized in the backend I'll keep them for now
interface Asset {
  id: number;
  name: string;
}

interface AssetCategory {
  name: string;
  color: string;
  assets: Asset[];
}

const assetCategories = ref<AssetCategory[]>([]);

// FIXME(nick,victor): temporary measure to populate the assetCategories dynamically based on the application.
ApplicationService.currentApplication().subscribe((application) => {
  if (application === null) {
    assetCategories.value = [];
    return;
  }

  SchematicService.getNodeAddMenu({
    menuFilter: {
      rootComponentId: application.id,
      schematicKind: SchematicKind.Component,
    },
  }).subscribe((response: ApiResponse<MenuItem[]>) => {
    if (response.error) {
      assetCategories.value = [];
      GlobalErrorService.set(response);
      return;
    }

    // TODO(victor): when the old interface goes, the API probably could return the expected structure and we won't need this conversion
    // for now, we assume the endpoint returns an array of `api/sdf/dal/menu.Category` containing an array of `api/sdf/dal/menu.Item`
    assetCategories.value = response
      .filter((c) => c.kind === "category")
      .map((c) => {
        const { name, items } = c as Category;
        return {
          name,
          color: "#00F", // TODO(victor) refactor menu endpoint to send color info
          assets: items
            .filter((i) => i.kind === "item")
            .map((i) => {
              const { name, schema_id: id } = i as Item;
              return {
                name,
                id,
              };
            }),
        };
      });
  });
});

const activeNode = ref<number | undefined>();

const setActiveNode = (e: Asset, _event: MouseEvent) => {
  // TODO(victor): This code makes it so that clicking the selected node deselects it. That should probably change when node addiction is handled by an observable
  activeNode.value = e.id !== activeNode.value ? e.id : undefined;

  // TODO(nick): temporarily embedding the add node into the active node event.
  addNode(e.id, _event);
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
  // TODO(victor) we should subscribe to viewerEvents to sync the selected node with the real state
};
</script>
