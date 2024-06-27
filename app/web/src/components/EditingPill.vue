<template>
  <div
    class="p-2xs m-0 rounded text-xs font-bold"
    :style="{
      backgroundColor: $props.color,
      color: `#${bodyBg.toHex()}`,
    }"
  >
    EDITING
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import tinycolor from "tinycolor2";
import { useTheme } from "@si/vue-lib/design-system";

const { theme } = useTheme();

const props = defineProps({
  color: { type: String },
});

const primaryColor = tinycolor(props.color ?? "000000");

const bodyBg = computed(() => {
  const bodyBgHsl = primaryColor.toHsl();
  bodyBgHsl.l = theme.value === "dark" ? 0.08 : 0.95;
  return tinycolor(bodyBgHsl);
});
</script>
