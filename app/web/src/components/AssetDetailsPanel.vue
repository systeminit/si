<template>
  <div class="grow relative">
    <RequestStatusMessage
      v-if="loadAssetReqStatus.isPending"
      :requestStatus="loadAssetReqStatus"
    />
    <ScrollArea v-else-if="editingAsset && props.assetId">
      <template #top>
        <div
          class="flex flex-row items-center justify-around gap-xs p-xs border-b dark:border-neutral-600"
        >
          <VButton
            :disabled="saveAssetReqStatus.isPending"
            :loading="updateAssetReqStatus.isPending"
            :requestStatus="updateAssetReqStatus"
            icon="bolt"
            label="Regenerate Asset"
            loadingText="Regenerating Asset..."
            size="md"
            successText="Successful"
            tone="action"
            @click="executeAsset"
          />
          <VButton
            icon="clipboard-copy"
            label="Clone"
            size="md"
            tone="neutral"
            @click="() => cloneAssetModalRef?.modal?.open()"
          />
        </div>
        <AssetNameModal
          ref="cloneAssetModalRef"
          buttonLabel="Clone Asset"
          title="Asset Name"
          @submit="cloneAsset"
        />

        <ErrorMessage
          v-for="(warning, index) in assetStore.detachmentWarnings"
          :key="warning.message"
          :class="{ 'cursor-pointer': !!warning.kind }"
          class="mx-1"
          icon="alert-triangle"
          tone="warning"
          @click="openAttachModal(warning)"
        >
          {{ warning.message }}
          <VButton
            buttonRank="tertiary"
            icon="trash"
            size="xs"
            tone="destructive"
            @click.stop="assetStore.detachmentWarnings.splice(index, 1)"
          />
        </ErrorMessage>

        <AssetFuncAttachModal ref="attachModalRef" :assetId="props.assetId" />
      </template>

      <Stack class="p-xs" spacing="none">
        <div>
          <ErrorMessage :requestStatus="updateAssetReqStatus" />
        </div>
        <VormInput
          id="schemaName"
          v-model="editingAsset.schemaName"
          :disabled="ffStore.IMMUTABLE_SCHEMA_VARIANTS && editingAsset.isLocked"
          compact
          label="Asset Name"
          placeholder="(mandatory) Provide the asset a name"
          type="text"
          @blur="updateAsset"
        />

        <VormInput
          id="displayName"
          v-model="editingAsset.displayName"
          :disabled="ffStore.IMMUTABLE_SCHEMA_VARIANTS && editingAsset.isLocked"
          compact
          label="Display name"
          placeholder="(optional) Provide the asset version a display name"
          type="text"
          @blur="updateAsset"
        />
        <VormInput
          id="category"
          v-model="editingAsset.category"
          :disabled="ffStore.IMMUTABLE_SCHEMA_VARIANTS && editingAsset.isLocked"
          compact
          label="Category"
          placeholder="(mandatory) Provide a category for the asset"
          type="text"
          @blur="updateAsset"
        />
        <VormInput
          id="componentType"
          v-model="editingAsset.componentType"
          :disabled="ffStore.IMMUTABLE_SCHEMA_VARIANTS && editingAsset.isLocked"
          :options="componentTypeOptions"
          compact
          label="Component Type"
          type="dropdown"
          @change="updateAsset"
        />
        <VormInput
          id="description"
          v-model="editingAsset.description"
          :disabled="ffStore.IMMUTABLE_SCHEMA_VARIANTS && editingAsset.isLocked"
          compact
          label="Description"
          placeholder="(optional) Provide a brief description of the asset"
          type="textarea"
          @blur="updateAsset"
        />
        <VormInput
          :disabled="ffStore.IMMUTABLE_SCHEMA_VARIANTS && editingAsset.isLocked"
          compact
          label="color"
          type="container"
        >
          <ColorPicker
            id="color"
            v-model="editingAsset.color"
            @change="updateAsset"
          />
        </VormInput>

        <VormInput
          id="link"
          v-model="editingAsset.link"
          :disabled="ffStore.IMMUTABLE_SCHEMA_VARIANTS && editingAsset.isLocked"
          compact
          label="Documentation Link"
          placeholder="(optional) Provide a documentation link for the asset"
          type="url"
          @blur="updateAsset"
        />
      </Stack>
    </ScrollArea>
    <div
      v-else
      class="px-2 py-sm text-center text-neutral-400 dark:text-neutral-300"
    >
      <template v-if="props.assetId && loadAssetReqStatus.isError"
        >Asset "{{ props.assetId }}" does not exist!
      </template>
      <template v-else>Select an asset to view its details.</template>
    </div>
    <Modal
      ref="executeAssetModalRef"
      :title="
        editingAsset && editingAsset.schemaVariantId
          ? 'Asset Updated'
          : 'New Asset Created'
      "
      size="sm"
      @closeComplete="closeHandler"
    >
      {{
        editingAsset && editingAsset.schemaVariantId
          ? "The asset you just updated will be available to use from the Assets Panel"
          : "The asset you just created will now appear in the Assets Panel."
      }}
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import {
  ErrorMessage,
  Modal,
  RequestStatusMessage,
  ScrollArea,
  Stack,
  VButton,
  VormInput,
} from "@si/vue-lib/design-system";
import * as _ from "lodash-es";
import { FuncKind, FuncId } from "@/api/sdf/dal/func";
import { useAssetStore } from "@/store/asset.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { useFuncStore } from "@/store/func/funcs.store";
import { ComponentType } from "@/api/sdf/dal/schema";
import ColorPicker from "./ColorPicker.vue";
import AssetFuncAttachModal from "./AssetFuncAttachModal.vue";
import AssetNameModal from "./AssetNameModal.vue";

const props = defineProps<{
  assetId?: string;
}>();

const ffStore = useFeatureFlagsStore();
const assetStore = useAssetStore();
const funcStore = useFuncStore();
const loadAssetReqStatus = assetStore.getRequestStatus(
  "LOAD_SCHEMA_VARIANT",
  props.assetId,
);
const executeAssetModalRef = ref();
const cloneAssetModalRef = ref<InstanceType<typeof AssetNameModal>>();

const openAttachModal = (warning: { kind?: FuncKind; funcId?: FuncId }) => {
  if (!warning.kind) return;
  attachModalRef.value?.open(true, warning.kind, warning.funcId);
};

const componentTypeOptions = [
  {
    label: "Aggregation Frame",
    value: ComponentType.AggregationFrame,
  },
  { label: "Component", value: ComponentType.Component },
  {
    label: "Configuration Frame (down)",
    value: ComponentType.ConfigurationFrameDown,
  },
  {
    label: "Configuration Frame (up)",
    value: ComponentType.ConfigurationFrameUp,
  },
];

const attachModalRef = ref<InstanceType<typeof AssetFuncAttachModal>>();

const editingAsset = ref(_.cloneDeep(assetStore.selectedSchemaVariant));
watch(
  () => assetStore.selectedSchemaVariant,
  () => {
    editingAsset.value = _.cloneDeep(assetStore.selectedSchemaVariant);
  },
  { deep: true },
);

const updateAsset = async () => {
  if (
    editingAsset.value &&
    !_.isEqual(editingAsset.value, assetStore.selectedSchemaVariant)
  ) {
    const code =
      funcStore.funcDetailsById[editingAsset.value.assetFuncId]?.code;
    if (code) await assetStore.SAVE_SCHEMA_VARIANT(editingAsset.value);
    else
      throw new Error(
        `${editingAsset.value.assetFuncId} Func not found on Variant ${editingAsset.value.schemaVariantId}. This should not happen.`,
      );
  }
};

const updateAssetReqStatus = assetStore.getRequestStatus(
  "REGENERATE_VARIANT",
  assetStore.selectedVariantId,
);
const saveAssetReqStatus = assetStore.getRequestStatus(
  "SAVE_SCHEMA_VARIANT",
  assetStore.selectedVariantId,
);
const executeAsset = async () => {
  if (editingAsset.value) {
    await assetStore.REGENERATE_VARIANT(editingAsset.value.schemaVariantId);
  }
};

const closeHandler = () => {
  assetStore.executeSchemaVariantTaskId = undefined;
};

const cloneAsset = async (name: string) => {
  if (editingAsset.value?.schemaVariantId) {
    const result = await assetStore.CLONE_VARIANT(
      editingAsset.value.schemaVariantId,
      name,
    );
    if (result.result.success) {
      cloneAssetModalRef.value?.modal?.close();
      await assetStore.setSchemaVariantSelection(result.result.data.id);
    }
  }
};
</script>
