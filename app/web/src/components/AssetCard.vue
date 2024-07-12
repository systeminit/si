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
            class="hover:scale-125"
            icon="cloud-upload"
            tooltip="Contribute"
            tooltipPlacement="top"
            variant="simple"
          />

          <IconButton
            v-if="canUpdate"
            class="hover:scale-125"
            icon="code-deployed"
            tooltip="Update"
            tooltipPlacement="top"
            variant="simple"
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
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType, ref } from "vue";
import tinycolor from "tinycolor2";
import clsx from "clsx";
import { useTheme, Stack, Icon, ErrorMessage } from "@si/vue-lib/design-system";
import { useAssetStore } from "@/store/asset.store";
import { SchemaVariantId, SchemaVariant } from "@/api/sdf/dal/schema";
import { getAssetIcon } from "@/store/components.store";
import IconButton from "./IconButton.vue";
import EditingPill from "./EditingPill.vue";

const props = defineProps({
  titleCard: { type: Boolean },
  assetId: { type: String as PropType<SchemaVariantId>, required: true },
});

const { theme } = useTheme();

const editingVersionDoesNotExist = computed<boolean>(
  () =>
    assetStore.unlockedVariantIdForId[asset.value?.schemaVariantId ?? ""] ===
    undefined,
);

const assetStore = useAssetStore();
const asset = computed(
  (): SchemaVariant | undefined =>
    assetStore.variantFromListById[props.assetId],
);

const canUpdate = computed(
  () => !!assetStore.upgradeableModules[props.assetId],
);

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
</script>
