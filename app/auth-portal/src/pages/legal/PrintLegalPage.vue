<template>
  <div class="p-lg bg-white text-black">
    <RichText>
      <Component :is="LEGAL_DOCS_CONTENT[urlDocSlug].component" />
    </RichText>
  </div>
</template>

<script setup lang="ts">
import { RichText } from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { onMounted } from "vue";
import { useRoute } from "vue-router";

import { LEGAL_DOCS_CONTENT } from "./load-docs";

const route = useRoute();
const urlDocVersion = route.params.docVersion as string;
const urlDocSlug = route.params.docSlug as string;

onMounted(triggerPrint);

function triggerPrint() {
  document.title = `system-initiative-${urlDocSlug}-${urlDocVersion}`
  window.print();
  window.onfocus = () => {
    window.close();
  };
}

useHead({
  htmlAttrs: {
    style: "color-scheme: light;",
    class: "light",
  },
});
</script>
