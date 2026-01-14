<template>
  <div>
    <ScrollArea>
      <template #top>
        <SidebarSubpanelTitle class="mt-2xs" icon="func" label="Asset Functions">
          <AssetFuncAttachDropdown
            v-if="assetStore.selectedVariantId"
            :disabled="!assetStore.selectedSchemaVariant?.schemaVariantId"
            @selected-attach-type="openAttachFuncModal"
          />
        </SidebarSubpanelTitle>
      </template>

      <FuncList
        v-if="assetStore.selectedVariantId"
        :funcsByKind="funcsByKindWithDeprecationFiltering"
        context="workspace-lab-assets"
        defaultOpen
      />
      <template v-else>
        <EmptyStateCard
          v-if="assetStore.selectedSchemaVariants.length > 1"
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
    <AssetFuncAttachModal ref="attachModalRef" :schemaVariantId="schemaVariantId" />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref } from "vue";
import groupBy from "lodash-es/groupBy";
import { ScrollArea } from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import { FuncKind, FuncSummary } from "@/api/sdf/dal/func";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import { SchemaVariantId, SchemaVariant } from "@/api/sdf/dal/schema";
import AssetFuncAttachModal from "./AssetFuncAttachModal.vue";
import AssetFuncAttachDropdown from "./AssetFuncAttachDropdown.vue";
import FuncList from "./FuncEditor/FuncList.vue";
import EmptyStateCard from "./EmptyStateCard.vue";

const props = defineProps<{ schemaVariantId?: SchemaVariantId }>();

const assetStore = useAssetStore();
const funcStore = useFuncStore();

interface VariantsWithFunctionSummary extends SchemaVariant {
  funcSummaries: FuncSummary[];
  assetFunc: FuncSummary;
}

const variantSummaries = computed(() => {
  if (!props.schemaVariantId) return null;

  const variant = _.cloneDeep(assetStore.variantFromListById[props.schemaVariantId]) as VariantsWithFunctionSummary;
  if (!variant) return null;
  variant.funcSummaries = [];

  // seeing duplicates, can prevent that from this end
  [...new Set(variant.funcIds)].forEach((fId) => {
    const func = funcStore.funcsById[fId];
    if (func) variant.funcSummaries.push(func);
  });
  const func = funcStore.funcsById[variant.assetFuncId];
  if (func) variant.assetFunc = func;
  return variant;
});

const funcsByKindWithDeprecationFiltering = computed(() => {
  const r = variantSummaries.value ? groupBy(variantSummaries.value.funcSummaries ?? [], (f) => f.kind) : {};

  // NOTE(nick): filter out deprecated attribute funcs that have become intrinsic funcs.
  if (r[FuncKind.Attribute]) {
    r[FuncKind.Attribute] = r[FuncKind.Attribute].filter((f) => {
      if (f.name === "si:resourcePayloadToValue") return false;
      else if (f.name === "si:normalizeToArray") return false;
      return true;
    });
  }

  return r;
});

const attachModalRef = ref<InstanceType<typeof AssetFuncAttachModal>>();

const openAttachFuncModal = (type: "new" | "existing") => {
  if (type === "new") {
    attachModalRef.value?.open(false);
  } else {
    attachModalRef.value?.open(true);
  }
};
</script>
