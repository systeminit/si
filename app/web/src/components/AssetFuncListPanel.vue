<template>
  <div>
    <RequestStatusMessage
      v-if="loadAssetReqStatus.isPending"
      :requestStatus="loadAssetReqStatus"
      showLoaderWithoutMessage
    />
    <ScrollArea>
      <template #top>
        <SidebarSubpanelTitle class="border-t-0">
          <div class="flex flex-row items-center justify-between">
            <span class="pt-1">Asset Functions</span>
            <AssetFuncAttachDropdown
              v-if="assetStore.selectedAssetId && !changeSetsStore.headSelected"
              :disabled="!assetStore.selectedAsset?.schemaVariantId"
              label="Attach Function"
              @selected-attach-type="openAttachFuncModal"
            />
          </div>
        </SidebarSubpanelTitle>
        <div
          v-if="!assetStore.selectedAssetId"
          class="w-full mt-4 p-sm text-neutral-400 dark:text-neutral-300 text-sm text-center"
        >
          Select an asset to see the functions attached to it.
        </div>
      </template>

      <ul
        v-if="assetStore.selectedAssetId && !loadAssetReqStatus.isPending"
        class="overflow-y-auto min-h-[200px]"
      >
        <Collapsible
          v-for="(label, variant) in CUSTOMIZABLE_FUNC_TYPES"
          :key="variant"
          as="li"
          class="w-full"
          contentAs="ul"
          defaultOpen
        >
          <template #label>
            <div class="flex items-center gap-2">
              <FuncSkeleton />
              <span> {{ label.pluralLabel }} </span>
            </div>
          </template>

          <template #default>
            <li v-for="func in funcsByVariant[variant] ?? []" :key="func.id">
              <SiFuncListItem
                :func="func"
                color="#921ed6"
                context="workspace-lab-assets"
              />
            </li>
          </template>
        </Collapsible>
      </ul>
    </ScrollArea>
    <AssetFuncAttachModal
      ref="attachModalRef"
      :schemaVariantId="assetSchemaVariantId"
      :assetId="assetId"
    />
  </div>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import groupBy from "lodash-es/groupBy";
import {
  Collapsible,
  ScrollArea,
  RequestStatusMessage,
} from "@si/vue-lib/design-system";
import { CUSTOMIZABLE_FUNC_TYPES } from "@/api/sdf/dal/func";
import { useAssetStore } from "@/store/asset.store";
import SiFuncListItem from "@/components/SiFuncListItem.vue";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import FuncSkeleton from "./FuncSkeleton.vue";
import AssetFuncAttachModal from "./AssetFuncAttachModal.vue";
import AssetFuncAttachDropdown from "./AssetFuncAttachDropdown.vue";

const props = defineProps<{ assetId?: string }>();

const changeSetsStore = useChangeSetsStore();
const assetStore = useAssetStore();

const funcsByVariant = computed(() =>
  props.assetId
    ? groupBy(
        assetStore.assetsById[props.assetId]?.funcs ?? [],
        (f) => f.variant,
      )
    : {},
);

const loadAssetReqStatus = assetStore.getRequestStatus(
  "LOAD_ASSET",
  props.assetId,
);

const attachModalRef = ref<InstanceType<typeof AssetFuncAttachModal>>();
const assetSchemaVariantId = computed(() =>
  props.assetId
    ? assetStore.assetsById[props.assetId]?.schemaVariantId
    : undefined,
);

const openAttachFuncModal = (type: "new" | "existing") => {
  if (type === "new") {
    attachModalRef.value?.open(false);
  } else {
    attachModalRef.value?.open(true);
  }
};
</script>
