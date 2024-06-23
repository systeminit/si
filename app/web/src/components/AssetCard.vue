<template>
  <div
    v-if="asset"
    :class="
      clsx(
        'p-xs border-l-4 border relative',
        titleCard ? 'mb-xs' : 'rounded-md',
      )
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
      <div
        v-tooltip="{
          content: 'Upgrade',
          theme: 'instant-show',
        }"
        class="ml-auto cursor-pointer flex flex-none gap-xs"
      >
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
      </div>

      <!-- Slot for additional icons/buttons -->
      <slot />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType, ref } from "vue";
import tinycolor from "tinycolor2";
import clsx from "clsx";
import { useTheme, Stack, Icon } from "@si/vue-lib/design-system";
import { useAssetStore, AssetListEntry, AssetId } from "@/store/asset.store";
import { getAssetIcon } from "@/store/components.store";
import IconButton from "./IconButton.vue";

const props = defineProps({
  titleCard: { type: Boolean },
  assetId: { type: String as PropType<AssetId>, required: true },
});

const { theme } = useTheme();

const assetStore = useAssetStore();
const asset = computed(
  (): AssetListEntry | undefined => assetStore.assetFromListById[props.assetId],
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
</script>
