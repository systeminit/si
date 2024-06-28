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
        <Icon :name="getAssetIcon(asset.category)" size="lg" class="shrink-0" />
        <Stack spacing="xs" class="">
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
            variant="simple"
            icon="cloud-upload"
            tooltip="Contribute"
            tooltipPlacement="top"
          />

          <IconButton
            v-if="asset.canUpdate"
            class="hover:scale-125"
            variant="simple"
            icon="code-deployed"
            tooltip="Update"
            tooltipPlacement="top"
          />

          <IconButton
            v-if="
              ffStore.IMMUTABLE_SCHEMA_VARIANTS &&
              asset.isLocked &&
              editingVersionDoesNotExist
            "
            class="hover:scale-125"
            variant="simple"
            icon="sliders-vertical"
            tooltip="Edit"
            tooltipPlacement="top"
            @click="unlock"
          />
          <Icon v-if="!asset.isLocked" tone="action" name="sliders-vertical" />
        </div>

        <!-- Slot for additional icons/buttons -->
        <slot />
      </div>
    </div>
    <div class="flex flex-col">
      <ErrorMessage
        v-if="asset && ffStore.IMMUTABLE_SCHEMA_VARIANTS && asset.isLocked"
        icon="lock"
        variant="block"
        tone="warning"
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
import { useAssetStore, SchemaVariantListEntry } from "@/store/asset.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { SchemaVariantId } from "@/api/sdf/dal/schema";
import { getAssetIcon } from "@/store/components.store";
import IconButton from "./IconButton.vue";
import EditingPill from "./EditingPill.vue";

const props = defineProps({
  titleCard: { type: Boolean },
  assetId: { type: String as PropType<SchemaVariantId>, required: true },
});

const { theme } = useTheme();
const ffStore = useFeatureFlagsStore();

const editingVersionDoesNotExist = computed<boolean>(() => {
  const unlockedExists = assetStore.variantList.some(
    (v) => v.schemaId === asset.value?.schemaId && !v.isLocked,
  );
  return !unlockedExists;
});

const assetStore = useAssetStore();
const asset = computed(
  (): SchemaVariantListEntry | undefined =>
    assetStore.variantFromListById[props.assetId],
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
      assetStore.setSchemaVariantSelection(resp.result.data?.id);
    }
  }
};
</script>
