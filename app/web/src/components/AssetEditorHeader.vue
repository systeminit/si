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
        v-if="generateAwsFunction.available"
        icon="sparkles"
        size="lg"
        :tooltip="`Generate ${generateAwsFunction.kind?.description}`"
        :selected="generateAwsFunction.visible"
        @click="generateAwsFunction.toggle()"
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
      <div v-show="generateAwsFunction.visible" ref="transitionRef">
        <GenerateAwsFunctionPanel
          :funcId="funcId"
          :schemaVariantId="selectedAsset.schemaVariantId"
          :kind="generateAwsFunction.kind as GenerateAwsFunctionKind"
          :isLocked="selectedAsset?.isLocked || selectedFunc?.isLocked"
        />
      </div>
    </Transition>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import {
  IconButton,
  Timestamp,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { schemaVariantDisplayName, useAssetStore } from "@/store/asset.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import {
  GenerateAwsFunctionKind,
  GenerateAwsFunctionKinds,
  useFuncStore,
} from "@/store/func/funcs.store";
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

const props = defineProps<{
  selectedAsset: SchemaVariant;
  selectedFunc?: FuncSummary;
}>();

const onClick = () => {
  assetStore.setFuncSelection(undefined);
};

const showTooltip = computed(() => {
  return truncateRef1.value?.tooltipActive || truncateRef2.value?.tooltipActive;
});

const funcId = computed(
  () => props.selectedFunc?.funcId ?? props.selectedAsset.assetFuncId,
);

const generateAwsFunction = {
  /** The kind of function we're generating */
  get kind() {
    if (!props.selectedFunc) return GenerateAwsFunctionKinds.AssetSchema;
    switch (props.selectedFunc.kind) {
      case "Action":
        return GenerateAwsFunctionKinds.Action;
      case "Management":
        return GenerateAwsFunctionKinds.Management;
      default:
        return undefined;
    }
  },
  /** Whether this type of function can actually be generated and is unlocked */
  get available() {
    return featureFlagsStore.AI_GENERATOR && this.kind;
  },
  /** Whether the AI generation panel should be visible */
  get visible() {
    return (
      this.available &&
      (funcStore.generateAwsFunctionPanelToggled ||
        !!funcStore.generatingFuncCode[funcId.value])
    );
  },
  /** Toggle visibility */
  toggle() {
    funcStore.generateAwsFunctionPanelToggled = !this.visible;
  },
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
