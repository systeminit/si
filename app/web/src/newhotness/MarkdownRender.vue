<template>
  <div class="prose dark:prose-invert">
    <VueMarkdown
      :source="props.source"
      :options="{ breaks: true, linkify: true, typographer: true }"
    />
  </div>
</template>

<script setup lang="ts">
import VueMarkdown from "vue-markdown-render";

const props = defineProps<{ source: string }>();
</script>
