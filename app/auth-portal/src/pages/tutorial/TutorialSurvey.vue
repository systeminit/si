<template>
  <div>
    <iframe
      class="my-0 mx-auto"
      :src="iframeSrc"
      :width="width"
      :height="height"
      frameborder="0"
      marginheight="0"
      marginwidth="0"
    >
      {{ loadingText }}
    </iframe>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";

const props = defineProps({
  surveyId: { type: String, required: true },
  loadingText: { type: String, default: "Loading..." },
  width: { type: Number, default: 640 },
  height: { type: Number, default: 1800 },
});

const iframeSrc = computed(() => {
  return `https://docs.google.com/forms/d/e/${props.surveyId}/viewform?embedded=true`;
});
</script>
