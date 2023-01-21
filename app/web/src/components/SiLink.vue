<template>
  <a :href="uri" :target="target" :rel="rel">
    <slot />
  </a>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  uri: string;
  blankTarget?: boolean;
}>();

const target = computed(() => {
  if (props.blankTarget) return "_blank";
  return "_self";
});

const rel = computed(() => {
  // There is some danger in using target _blank for untrusted links
  // Check this for more information on rel: https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes/rel
  //
  // nofollow is an SEO thing: https://en.wikipedia.org/wiki/Nofollow
  if (props.blankTarget) return "noreferrer nofollow";
  return undefined;
});
</script>

<style scoped></style>
