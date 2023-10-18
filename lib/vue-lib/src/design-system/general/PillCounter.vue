<template>
  <div
    v-if="count !== 0 || showIfZero"
    class="inline-block rounded-2xl px-xs border border-current dark:text-neutral-300 text-neutral-600"
  >
    {{ count }}
  </div>
</template>

<script setup lang="ts">
import { PropType } from "vue";
import { Tones } from "../utils/color_utils";

const props = defineProps({
  count: Number,
  // TODO: implement color/tone options
  tone: { type: String as PropType<Tones>, default: "success" },
  showIfZero: { type: Boolean },
});
</script>
