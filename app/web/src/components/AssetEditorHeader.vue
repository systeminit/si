<template>
  <div class="p-xs">
    <TruncateWithTooltip
      class="text-2xl font-bold pb-2xs flex flex-row items-center gap-xs"
    >
      <div
        :class="
          clsx(
            'flex flex-row items-center gap-xs',
            selectedFunc && [
              'cursor-pointer hover:underline',
              themeClasses('text-action-500', 'text-action-300'),
            ],
          )
        "
        @click="onClick"
      >
        <NodeSkeleton :color="selectedAsset.color" size="mini" />
        {{ schemaVariantDisplayName(selectedAsset) }}
      </div>
      <div v-if="selectedFunc">
        / {{ selectedFunc.kind }} / {{ selectedFunc.name }}
      </div>
    </TruncateWithTooltip>
    <EditingPill
      v-if="!selectedAsset.isLocked"
      class="mt-2xs"
      :color="selectedAsset.color"
    />
    <div
      class="text-xs italic flex flex-row flex-wrap gap-x-lg text-neutral-600 dark:text-neutral-200"
    >
      <div>
        <span class="font-bold">Created At: </span>
        <Timestamp :date="selectedAsset.created_at" size="long" />
      </div>
      <!-- TODO: Populate the created by from SDF actorHistory-->
      <div><span class="font-bold">Created By: </span>System Initiative</div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import { Timestamp, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { schemaVariantDisplayName, useAssetStore } from "@/store/asset.store";
import { SchemaVariant } from "@/api/sdf/dal/schema";
import { FuncSummary } from "@/api/sdf/dal/func";
import EditingPill from "./EditingPill.vue";
import TruncateWithTooltip from "./TruncateWithTooltip.vue";
import NodeSkeleton from "./NodeSkeleton.vue";

const assetStore = useAssetStore();

defineProps({
  selectedAsset: { type: Object as PropType<SchemaVariant>, required: true },
  selectedFunc: { type: Object as PropType<FuncSummary> },
});

const onClick = () => {
  assetStore.setFuncSelection(undefined);
};
</script>
