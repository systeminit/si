<template>
  <div>
    <RequestStatusMessage
      v-if="loadAssetReqStatus.isPending"
      :requestStatus="loadAssetReqStatus"
    />
    <ScrollArea>
      <template #top>
        <SidebarSubpanelTitle
          label="Asset Functions"
          icon="func"
          class="mt-2xs"
        >
          <AssetFuncAttachDropdown
            v-if="assetStore.selectedVariantId"
            :disabled="!assetStore.selectedSchemaVariant?.schemaVariantId"
            @selected-attach-type="openAttachFuncModal"
          />
        </SidebarSubpanelTitle>
      </template>

      <FuncList
        v-if="assetStore.selectedVariantId && !loadAssetReqStatus.isPending"
        :funcsByKind="funcsByKind"
        context="workspace-lab-assets"
        defaultOpen
      />
      <template v-else>
        <EmptyStateCard
          v-if="assetStore.selectedAssets.length > 1"
          iconName="funcs"
          primaryText="Cannot Select Functions"
          secondaryText="You have selected multiple assets above. To select a function, select one asset."
        />
        <EmptyStateCard
          v-else
          iconName="funcs"
          primaryText="Select Asset, View Functions"
          secondaryText="Select an asset from the list above to view its attached functions here."
        />
      </template>
    </ScrollArea>
    <AssetFuncAttachModal ref="attachModalRef" :assetId="assetId" />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref } from "vue";
import groupBy from "lodash-es/groupBy";
import { RequestStatusMessage, ScrollArea } from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { FuncSummary, useFuncStore } from "@/store/func/funcs.store";
import { SchemaVariant } from "@/api/sdf/dal/schema";
import { FuncId } from "@/api/sdf/dal/func";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import AssetFuncAttachModal from "./AssetFuncAttachModal.vue";
import AssetFuncAttachDropdown from "./AssetFuncAttachDropdown.vue";
import FuncList from "./FuncEditor/FuncList.vue";
import EmptyStateCard from "./EmptyStateCard.vue";

const props = defineProps<{ assetId?: string }>();

const assetStore = useAssetStore();
const funcStore = useFuncStore();

interface VariantsWithFunctionSummary extends SchemaVariant {
  funcSummaries: FuncSummary[];
  assetFunc: FuncSummary;
}

const variantSummaries = computed(() => {
  if (!props.assetId) return null;

  const variant = assetStore.variantsById[
    props.assetId
  ] as VariantsWithFunctionSummary;
  variant.funcs.forEach((fId) => {
    const func = funcStore.funcsById[fId];
    if (func) variant.funcSummaries.push(func);
  });
  const func = funcStore.funcsById[variant.assetFuncId];
  if (func) variant.assetFunc = func;
  return variant;
});

const funcsByKind = computed(() =>
  variantSummaries.value
    ? groupBy(variantSummaries.value.funcSummaries ?? [], (f) => f.kind)
    : {},
);

const loadAssetReqStatus = assetStore.getRequestStatus(
  "LOAD_SCHEMA_VARIANT",
  props.assetId,
);

const attachModalRef = ref<InstanceType<typeof AssetFuncAttachModal>>();

const openAttachFuncModal = (type: "new" | "existing") => {
  if (type === "new") {
    attachModalRef.value?.open(false);
  } else {
    attachModalRef.value?.open(true);
  }
};
</script>
