<template>
  <div>
    <div
      v-if="asset"
      :class="
        clsx('p-xs border-l-4 border relative', !titleCard && 'rounded-md')
      "
      :style="{
        borderColor: asset.color,
        backgroundColor: `#${bodyBg.toHex()}`,
      }"
    >
      <div class="flex gap-xs items-center">
        <Icon :name="getAssetIcon(asset.category)" class="shrink-0" size="lg" />
        <Stack class="" spacing="xs">
          <div
            ref="componentNameRef"
            v-tooltip="componentNameTooltip"
            class="font-bold break-all line-clamp-4 pb-[2px]"
          >
            <template v-if="asset.displayName">
              {{ asset.displayName }}
            </template>
            <template v-else>
              {{ asset.schemaName }}
            </template>
          </div>
        </Stack>

        <!-- ICONS AFTER THIS POINT ARE RIGHT ALIGNED DUE TO THE ml-auto STYLE ON THIS DIV -->
        <div class="ml-auto flex flex-none gap-xs">
          <EditingPill v-if="!asset.isLocked" :color="asset.color" />
          <IconButton
            v-if="asset.canContribute"
            :selected="contributeAssetModalRef?.isOpen || false"
            class="hover:scale-125"
            icon="cloud-upload"
            tooltip="Contribute"
            tooltipPlacement="top"
            variant="simple"
            @click="contributeAsset"
          />

          <IconButton
            v-if="canUpdate"
            class="hover:scale-125"
            icon="code-deployed"
            tooltip="Update"
            tooltipPlacement="top"
            variant="simple"
            @click="updateAsset"
          />

          <IconButton
            v-if="!asset.isLocked && hasEditingVersion"
            class="hover:scale-125"
            icon="trash"
            tooltip="Delete Unlocked Variant"
            tooltipPlacement="top"
            variant="simple"
            @click="deleteUnlockedVariant"
          />

          <IconButton
            v-if="asset.isLocked && editingVersionDoesNotExist"
            class="hover:scale-125"
            icon="sliders-vertical"
            tooltip="Edit"
            tooltipPlacement="top"
            variant="simple"
            @click="unlock"
          />
          <Icon v-if="!asset.isLocked" name="sliders-vertical" tone="action" />
        </div>

        <!-- Slot for additional icons/buttons -->
        <slot />
      </div>
    </div>
    <div class="flex flex-col">
      <ErrorMessage
        v-if="deleteUnlockedVariantReqStatus.isError"
        :requestStatus="deleteUnlockedVariantReqStatus"
        variant="block"
      />
      <ErrorMessage
        v-if="asset && asset.isLocked"
        icon="lock"
        tone="warning"
        variant="block"
      >
        <template v-if="editingVersionDoesNotExist">
          Click edit to create a new editable version of this asset.
        </template>
        <template v-else>
          An editable version of this asset exists. This version is locked.
        </template>
      </ErrorMessage>
    </div>
    <!-- FIXME(nick): this probably needs to be moved and de-duped with logic in AssetListPanel -->
    <AssetContributeModal
      ref="contributeAssetModalRef"
      :contributeRequest="contributeRequest"
      @contribute-success="onContributeAsset"
    />
    <Modal
      ref="contributeAssetSuccessModalRef"
      size="sm"
      title="Contribution sent"
    >
      <p>
        Thanks for contributing! We will review your contribution, and reach out
        via email or on our
        <a class="text-action-500" href="https://discord.com/invite/system-init"
          >Discord Server</a
        >
        if you have any questions.
      </p>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType, ref } from "vue";
import tinycolor from "tinycolor2";
import clsx from "clsx";
import {
  useTheme,
  Modal,
  Stack,
  Icon,
  ErrorMessage,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { format as dateFormat } from "date-fns";
import { useAssetStore } from "@/store/asset.store";
import { SchemaVariantId, SchemaVariant } from "@/api/sdf/dal/schema";
import { getAssetIcon } from "@/store/components.store";
import { useModuleStore } from "@/store/module.store";
import { ModuleContributeRequest } from "@/api/sdf/dal/module";
import IconButton from "./IconButton.vue";
import EditingPill from "./EditingPill.vue";
import AssetContributeModal from "./AssetContributeModal.vue";

const props = defineProps({
  titleCard: { type: Boolean },
  assetId: { type: String as PropType<SchemaVariantId>, required: true },
});

const assetStore = useAssetStore();
const moduleStore = useModuleStore();
const router = useRouter();
const { theme } = useTheme();

const contributeAssetModalRef =
  ref<InstanceType<typeof AssetContributeModal>>();
const contributeAssetSuccessModalRef = ref<InstanceType<typeof Modal>>();

const contributeAsset = () => contributeAssetModalRef.value?.open();
const onContributeAsset = () => contributeAssetSuccessModalRef.value?.open();

const contributeRequest = computed((): ModuleContributeRequest => {
  if (asset.value) {
    const version = dateFormat(Date.now(), "yyyyMMddkkmmss");
    return {
      name: `${asset.value.schemaName} ${version}`,
      version,
      schemaVariantId: asset.value.schemaVariantId,
    };
  } else throw new Error("cannot contribute: no asset selected");
});

const editingVersionDoesNotExist = computed<boolean>(
  () =>
    assetStore.unlockedVariantIdForId[asset.value?.schemaVariantId ?? ""] ===
    undefined,
);

const hasEditingVersion = computed<boolean>(
  () =>
    assetStore.unlockedVariantIdForId[asset.value?.schemaVariantId ?? ""] !==
    undefined,
);

const asset = computed(
  (): SchemaVariant | undefined =>
    assetStore.variantFromListById[props.assetId],
);

const canUpdate = computed(
  () => !!moduleStore.upgradeableModules[props.assetId],
);

const updateAsset = () => {
  const schemaVariantId = asset.value?.schemaVariantId;
  if (!schemaVariantId) {
    throw new Error("cannot update asset: no asset selected");
  }

  const module = moduleStore.upgradeableModules[schemaVariantId];
  if (!module) {
    throw new Error("cannot update asset: no upgradeable module for asset");
  }

  moduleStore.INSTALL_REMOTE_MODULE(module.id);
  router.replace({
    name: "workspace-lab-assets",
  });

  return;
};

const primaryColor = tinycolor(asset.value?.color ?? "000000");

// body bg
const bodyBg = computed(() => {
  const bodyBgHsl = primaryColor.toHsl();
  bodyBgHsl.l = theme.value === "dark" ? 0.08 : 0.95;
  return tinycolor(bodyBgHsl);
});

const componentNameRef = ref();
const componentNameTooltip = computed(() => {
  if (
    componentNameRef.value &&
    componentNameRef.value.scrollHeight > componentNameRef.value.offsetHeight
  ) {
    return {
      content: componentNameRef.value.textContent,
      delay: { show: 700, hide: 10 },
    };
  } else {
    return {};
  }
});

const unlock = async () => {
  if (asset.value) {
    const resp = await assetStore.CREATE_UNLOCKED_COPY(
      asset.value.schemaVariantId,
    );
    if (resp.result.success) {
      assetStore.setSchemaVariantSelection(resp.result.data?.schemaVariantId);
    }
  }
};

const deleteUnlockedVariantReqStatus = assetStore.getRequestStatus(
  "DELETE_UNLOCKED_VARIANT",
  asset.value?.schemaVariantId,
);
const deleteUnlockedVariant = async () => {
  if (asset.value) {
    const resp = await assetStore.DELETE_UNLOCKED_VARIANT(
      asset.value.schemaVariantId,
    );
    if (resp.result.success) {
      assetStore.setSchemaVariantSelection("");
      router.replace({
        name: "workspace-lab-assets",
      });
    }
  }
};
</script>
