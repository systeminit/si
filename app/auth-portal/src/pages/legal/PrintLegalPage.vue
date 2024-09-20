<template>
  <div class="p-lg bg-white text-black">
    <RichText>
      <Component
        :is="LEGAL_DOCS_CONTENT[urlDocVersion as TosVersion][urlDocSlug].component"
      />
    </RichText>
  </div>
</template>

<script lang="ts" setup>
import { RichText } from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { onMounted } from "vue";
import { useRoute } from "vue-router";

import { TosVersion } from "@si/ts-lib/src/terms-of-service";
import { LEGAL_DOCS_CONTENT } from "./load-docs";

const route = useRoute();
const urlDocVersion = route.params.docVersion as string;
const urlDocSlug = route.params.docSlug as string;

onMounted(triggerPrint);

function triggerPrint() {
  document.title = `system-initiative-${urlDocSlug}-${urlDocVersion}`;
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
