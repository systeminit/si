<template>
  <p
    class="border-b-2 dark:border-neutral-600 text-sm leading-tight p-2.5 text-neutral-500"
  >
    Drag the assets that you wish to include in your application into the canvas
    to the right.
  </p>

  <ul class="overflow-y-auto">
    <SiCollapsible
      v-for="(category, category_index) in assetCategories"
      :key="category_index"
      :label="category.name"
      as="li"
      content-as="ul"
      default-open
    >
      <li v-for="(node, node_index) in category.assets" :key="node_index">
        <SiNodeSprite
          :class="selectedSchemaId === node.id ? 'bg-action-500' : ''"
          :color="node.color"
          :name="node.name"
          class="border-b-2 dark:border-neutral-600 hover:bg-action-500 dark:text-white hover:text-white hover:cursor-pointer"
          @mousedown="onSelect(node.id)"
        />
      </li>
    </SiCollapsible>
  </ul>
</template>

<script lang="ts" setup>
import _ from "lodash";
import { ref } from "vue";
import { combineLatest, firstValueFrom } from "rxjs";
import { untilUnmounted } from "vuse-rx";
import SiNodeSprite from "@/molecules/SiNodeSprite.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import { DiagramService } from "@/service/diagram";
import DiagramService2 from "@/service/diagram2";
import { DiagramNodeTemplate } from "@/api/sdf/dal/diagram";
import { Category, Item } from "@/api/sdf/dal/menu";

export type SelectAssetEvent = {
  schemaId: number;
};

const emit = defineEmits<{
  (e: "select", selectAssetEvent: SelectAssetEvent): void;
}>();

// TODO(victor): this types shouldn't be here, but since we probably need to refactor the way nodes/assets are organized in the backend I'll keep them for now
interface Asset {
  id: number;
  name: string;
  template: DiagramNodeTemplate;
  color: string;
}

interface AssetCategory {
  name: string;
  assets: Asset[];
}

const assetCategories = ref<AssetCategory[]>([]);

// TODO: move this whole thing into diagram data service - also return the data without needing so many API calls
combineLatest([DiagramService2.observables.schemaVariants$])
  .pipe(untilUnmounted)
  .subscribe(async ([schemaVariants]) => {
    if (schemaVariants === null) {
      assetCategories.value = [];
      return;
    }

    const nodeAddMenu = await firstValueFrom(
      DiagramService.getNodeAddMenu({ diagramKind: "configuration" }),
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
              DiagramService.getNodeTemplate({ schemaId }),
            );

            if (template.error) continue;

            const variant = _.find(
              schemaVariants,
              (v) => v.id === template.schemaVariantId,
            );

            assets.push({
              name,
              id: schemaId,
              template,
              color: variant
                ? `#${variant.color.toString(16).padStart(6, "0")}`
                : "#777",
            });
          }

          return {
            name,
            assets,
          };
        }),
    );
  });

const selectedSchemaId = ref<number>();

function onSelect(schemaId: number) {
  selectedSchemaId.value = schemaId;
  emit("select", { schemaId });
}
</script>
