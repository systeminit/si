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
        v-if="!!generateAwsFunctionPanelKind"
        icon="sparkles"
        size="lg"
        :tooltip="generateAwsFunctionPanelTooltip"
        :selected="generateAwsFunctionPanelVisible"
        @click="toggleGenerateAwsFunctionPanel"
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
      <div v-show="generateAwsFunctionPanelVisible" ref="transitionRef">
        <GenerateAwsFunctionPanel
          :funcId="resolvedFuncId"
          :schemaVariantId="selectedAsset.schemaVariantId"
          :generatingCommand="generateAwsFunctionPanelGeneratingCommand"
        />
      </div>
    </Transition>
  </div>
</template>

<script lang="ts" setup>
import { PropType, computed, ref, watchEffect } from "vue";
import {
  IconButton,
  Timestamp,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { schemaVariantDisplayName, useAssetStore } from "@/store/asset.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { useFuncStore } from "@/store/func/funcs.store";
import { SchemaVariant } from "@/api/sdf/dal/schema";
import { FuncSummary } from "@/api/sdf/dal/func";
import EditingPill from "./EditingPill.vue";
import NodeSkeleton from "./NodeSkeleton.vue";
import GenerateAwsFunctionPanel from "./Ai/GenerateAwsFunctionPanel.vue";

const assetStore = useAssetStore();
const featureFlagsStore = useFeatureFlagsStore();
const funcStore = useFuncStore();

const truncateRef1 = ref<InstanceType<typeof TruncateWithTooltip>>();
const truncateRef2 = ref<InstanceType<typeof TruncateWithTooltip>>();

const props = defineProps({
  selectedAsset: { type: Object as PropType<SchemaVariant>, required: true },
  selectedFunc: { type: Object as PropType<FuncSummary> },
});

const onClick = () => {
  assetStore.setFuncSelection(undefined);
};

const showTooltip = computed(() => {
  return truncateRef1.value?.tooltipActive || truncateRef2.value?.tooltipActive;
});

// Generator panel button
const generateAwsFunctionPanelKind = computed(() => {
  if (!featureFlagsStore.AI_GENERATOR) {
    return undefined;
  }
  switch (props.selectedFunc?.kind) {
    case undefined:
      return "schema";
    case "Action":
      return "action";
    default:
      return undefined;
  }
});
const resolvedFuncId = computed(
  () => props.selectedFunc?.funcId ?? props.selectedAsset.assetFuncId,
);
const generateAwsFunctionPanelGeneratingCommand = computed(
  () => funcStore.generatingFuncCode[resolvedFuncId.value],
);
const generateAwsFunctionPanelTooltip = computed(() => {
  switch (generateAwsFunctionPanelKind.value) {
    case "schema":
      return "Generate AWS Asset Schema";
    case "action":
      return "Generate AWS Action Function";
    default:
      return undefined;
  }
});
const generateAwsFunctionPanelVisible = ref(false);
const toggleGenerateAwsFunctionPanel = () => {
  generateAwsFunctionPanelVisible.value =
    !generateAwsFunctionPanelVisible.value;
};
// When we start or stop generating, make sure the panel is toggled on/off (but let the user toggle it back off if they want)
watchEffect(() => {
  generateAwsFunctionPanelVisible.value =
    !!generateAwsFunctionPanelGeneratingCommand.value;
});

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
