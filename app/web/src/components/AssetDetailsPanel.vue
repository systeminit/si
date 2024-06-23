<template>
  <div class="grow relative">
    <RequestStatusMessage
      v-if="loadAssetReqStatus.isPending"
      :requestStatus="loadAssetReqStatus"
    />
    <ScrollArea v-else-if="editingAsset && props.assetId">
      <template #top>
        <div
          class="flex flex-row items-center gap-xs p-xs border-b dark:border-neutral-600"
        >
          <VButton
            v-if="useFeatureFlagsStore().IMMUTABLE_SCHEMA_VARIANTS"
            icon="clipboard-copy"
            label="Unlock"
            size="md"
            tone="neutral"
            @click="unlock"
          />

          <VButton
            :loading="execAssetReqStatus.isPending"
            :requestStatus="execAssetReqStatus"
            icon="bolt"
            label="Regenerate Asset"
            loadingText="Regenerating Asset..."
            size="md"
            successText="Successful"
            tone="action"
            @click="executeAsset"
          />
          <VButton
            label="Clone"
            tone="neutral"
            icon="clipboard-copy"
            size="md"
            @click="() => cloneAssetModalRef?.modal?.open()"
          />
        </div>
        <AssetNameModal
          ref="cloneAssetModalRef"
          title="Asset Name"
          buttonLabel="Clone Asset"
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
          <ErrorMessage :requestStatus="execAssetReqStatus" />
        </div>
        <VormInput
          id="schemaName"
          v-model="editingAsset.schemaName"
          type="text"
          label="Asset Name"
          compact
          placeholder="(mandatory) Provide the asset a name"
          @blur="updateAsset"
        />
        <VormInput
          id="name"
          v-model="editingAsset.name"
          type="text"
          label="Asset Version Name"
          compact
          placeholder="(mandatory) Provide the asset version a name"
          @blur="updateAsset"
        />

        <VormInput
          id="displayName"
          v-model="editingAsset.displayName"
          type="text"
          label="Display name"
          compact
          placeholder="(optional) Provide the asset version a display name"
          @blur="updateAsset"
        />
        <VormInput
          id="category"
          v-model="editingAsset.category"
          compact
          label="Category"
          placeholder="(mandatory) Provide a category for the asset"
          type="text"
          @blur="updateAsset"
        />
        <VormInput
          id="componentType"
          v-model="editingAsset.componentType"
          :options="componentTypeOptions"
          compact
          label="Component Type"
          type="dropdown"
          @change="updateAsset"
        />
        <VormInput
          id="description"
          v-model="editingAsset.description"
          compact
          label="Description"
          placeholder="(optional) Provide a brief description of the asset"
          type="textarea"
          @blur="updateAsset"
        />
        <VormInput compact label="color" type="container">
          <ColorPicker
            id="color"
            v-model="editingAsset.color"
            @change="updateAsset"
          />
        </VormInput>

        <VormInput
          id="link"
          v-model="editingAsset.link"
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
      <template v-if="props.assetId"
        >Asset "{{ props.assetId }}" does not exist!
      </template>
      <template v-else>Select an asset to view its details.</template>
    </div>
    <Modal
      ref="executeAssetModalRef"
      :title="
        editingAsset && editingAsset.id ? 'Asset Updated' : 'New Asset Created'
      "
      size="sm"
      @closeComplete="closeHandler"
    >
      {{
        editingAsset && editingAsset.id
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
import { FuncKind } from "@/api/sdf/dal/func";
import { useAssetStore } from "@/store/asset.store";
import { FuncId } from "@/store/func/funcs.store";
import { ComponentType } from "@/api/sdf/dal/diagram";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import ColorPicker from "./ColorPicker.vue";
import AssetFuncAttachModal from "./AssetFuncAttachModal.vue";
import AssetNameModal from "./AssetNameModal.vue";

const props = defineProps<{
  assetId?: string;
}>();

const assetStore = useAssetStore();
const loadAssetReqStatus = assetStore.getRequestStatus(
  "LOAD_ASSET",
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

const editingAsset = ref(_.cloneDeep(assetStore.selectedAsset));
watch(
  () => assetStore.selectedAsset,
  () => {
    editingAsset.value = _.cloneDeep(assetStore.selectedAsset);
  },
  { deep: true },
);

const updateAsset = async () => {
  if (
    editingAsset.value &&
    !_.isEqual(editingAsset.value, assetStore.selectedAsset)
  ) {
    await assetStore.SAVE_ASSET(editingAsset.value);
  }
};

const execAssetReqStatus = assetStore.getRequestStatus(
  "EXEC_ASSET",
  assetStore.selectedAssetId,
);
const executeAsset = async () => {
  if (assetStore.selectedAssetId) {
    await assetStore.EXEC_ASSET(assetStore.selectedAssetId);
  }
};

const unlock = async () => {
  if (assetStore.selectedAsset?.defaultSchemaVariantId) {
    await assetStore.CREATE_UNLOCKED_COPY(
      assetStore.selectedAsset?.defaultSchemaVariantId,
    );
  }
};

const closeHandler = () => {
  assetStore.executeAssetTaskId = undefined;
};

const cloneAsset = async (name: string) => {
  if (editingAsset.value?.id) {
    const result = await assetStore.CLONE_ASSET(editingAsset.value.id, name);
    if (result.result.success) {
      cloneAssetModalRef.value?.modal?.close();
      await assetStore.setAssetSelection(result.result.data.id);
    }
  }
};
</script>
