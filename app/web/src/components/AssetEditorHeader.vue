<template>
  <div class="flex flex-col flex-grow">
    <!-- Main asset header -->
    <div class="p-xs flex flex-row gap-xs items-center">
      <div class="flex flex-col min-w-0 flex-grow">
        <TruncateWithTooltip
          :showTooltip="showTooltip"
          class="text-2xl font-bold pb-2xs flex flex-row items-center gap-xs"
        >
          <div
            :class="
              clsx(
                'flex flex-row items-center gap-xs flex-none max-w-full',
                selectedFunc && [
                  'cursor-pointer hover:underline',
                  themeClasses('text-action-500', 'text-action-300'),
                ],
              )
            "
            @click="onClick"
          >
            <NodeSkeleton :color="selectedAsset.color" size="mini" />
            <TruncateWithTooltip
              ref="truncateRef1"
              hasParentTruncateWithTooltip
            >
              {{ schemaVariantDisplayName(selectedAsset) }}
            </TruncateWithTooltip>
          </div>
          <TruncateWithTooltip
            v-if="selectedFunc"
            ref="truncateRef2"
            hasParentTruncateWithTooltip
          >
            / {{ selectedFunc.kind }} / {{ selectedFunc.name }}
          </TruncateWithTooltip>
        </TruncateWithTooltip>
        <div
          class="text-xs italic flex flex-row flex-wrap gap-x-lg text-neutral-600 dark:text-neutral-200 items-center"
        >
          <div>
            <span class="font-bold">Asset Created At: </span>
            <Timestamp :date="selectedAsset.created_at" size="long" />
          </div>
          <!-- TODO: Populate the created by from SDF actorHistory-->
          <div>
            <span class="font-bold">Created By: </span>System Initiative
          </div>
        </div>
      </div>
      <EditingPill
        v-if="!selectedAsset.isLocked"
        class="flex-none"
        :color="selectedAsset.color"
      />
      <IconButton
        v-if="featureFlagsStore.AI_GENERATOR"
        icon="sparkles"
        size="lg"
        :tooltip="
          showAwsAssetSchemaGeneratorPanel
            ? 'Close'
            : 'Generate AWS Asset Schema'
        "
        :selected="showAwsAssetSchemaGeneratorPanel"
        @click="toggleAwsAssetSchemaGeneratorPanel"
      />
    </div>
    <!-- openable AI generator panel extension -->
    <Transition
      name="expand-height"
      enterActiveClass="transition-[height] overflow-hidden"
      leaveActiveClass="transition-[height] overflow-hidden"
      enterFromClass="!h-0"
      leaveToClass="!h-0"
      :onBeforeEnter="captureHeight"
      :onAfterEnter="clearHeight"
      :onBeforeLeave="captureHeight"
      :onAfterLeave="clearHeight"
    >
      <div v-show="showAwsAssetSchemaGeneratorPanel" ref="transitionRef">
        <GenerateAwsAssetSchemaPanel :asset="selectedAsset" />
      </div>
    </Transition>
  </div>
</template>

<script lang="ts" setup>
import { PropType, computed, ref } from "vue";
import {
  IconButton,
  Timestamp,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { schemaVariantDisplayName, useAssetStore } from "@/store/asset.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { SchemaVariant } from "@/api/sdf/dal/schema";
import { FuncSummary } from "@/api/sdf/dal/func";
import EditingPill from "./EditingPill.vue";
import NodeSkeleton from "./NodeSkeleton.vue";
import GenerateAwsAssetSchemaPanel from "./Ai/GenerateAwsAssetSchemaPanel.vue";

const assetStore = useAssetStore();
const featureFlagsStore = useFeatureFlagsStore();

const truncateRef1 = ref<InstanceType<typeof TruncateWithTooltip>>();
const truncateRef2 = ref<InstanceType<typeof TruncateWithTooltip>>();

defineProps({
  selectedAsset: { type: Object as PropType<SchemaVariant>, required: true },
  selectedFunc: { type: Object as PropType<FuncSummary> },
});

const onClick = () => {
  assetStore.setFuncSelection(undefined);
};

const showTooltip = computed(() => {
  return truncateRef1.value?.tooltipActive || truncateRef2.value?.tooltipActive;
});

const showAwsAssetSchemaGeneratorPanel = ref(false);
const toggleAwsAssetSchemaGeneratorPanel = () => {
  showAwsAssetSchemaGeneratorPanel.value =
    !showAwsAssetSchemaGeneratorPanel.value;
};

const transitionRef = ref<HTMLDivElement>();

const captureHeight = () => {
  if (transitionRef.value) {
    if (transitionRef.value.style.display === "none") {
      transitionRef.value.style.removeProperty("display");
    }
    transitionRef.value.style.height = `${transitionRef.value.clientHeight}px`;
  }
};
const clearHeight = () => {
  if (transitionRef.value) {
    transitionRef.value.style.height = "";
  }
};
</script>
