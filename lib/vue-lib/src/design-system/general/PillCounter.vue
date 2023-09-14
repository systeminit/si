<template>
  <div
    class="inline-block rounded-2xl px-2.5 border border-success-500 ui-selected:bg-success-500 ui-selected:text-neutral-900 text-success-500"
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
});
</script>
