<template>
  <div
    class="p-3xs m-0 rounded text-xs font-bold max-h-fit"
    :style="{
      backgroundColor: $props.color,
      color: `#${text.toHex()}`,
    }"
  >
    Working Copy
  </div>
</template>

<script lang="ts" setup>
import { computed, watch, ref } from "vue";
import tinycolor from "tinycolor2";

const props = defineProps({
  color: { type: String },
});

const primaryColor = ref(tinycolor("000000"));
watch(
  () => props.color,
  () => {
    primaryColor.value = tinycolor(props.color ?? "000000");
  },
  { immediate: true },
);

const text = computed(() => {
  const textBgHsl = primaryColor.value.toHsl();
  textBgHsl.l = textBgHsl.l > 0.5 ? 0 : 1;
  return tinycolor(textBgHsl);
});
</script>
