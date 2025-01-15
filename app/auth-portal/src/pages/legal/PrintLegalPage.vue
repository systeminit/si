<template>
  <div class="p-lg bg-shade-0 text-shade-100">
    <RichText>
      <Component :is="thisDoc.component" />
    </RichText>
  </div>
</template>

<script lang="ts" setup>
import { RichText } from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { computed, onMounted } from "vue";
import { useRoute } from "vue-router";

import { TosVersion } from "@si/ts-lib/src/terms-of-service";
import { LEGAL_DOCS_CONTENT } from "./load-docs";

const route = useRoute();
const urlDocVersion = route.params.docVersion as string;
const urlDocSlug = route.params.docSlug as string;

const thisDoc = computed(
  () => LEGAL_DOCS_CONTENT[urlDocVersion as TosVersion][urlDocSlug],
);

const setPrintTitle = () => {
  const title = thisDoc.value.title.replaceAll(" ", "-");
  document.title = `${urlDocVersion}_SI-${title}`;
};

onMounted(() => {
  window.addEventListener("beforeprint", setPrintTitle);

  // Load bearing gambiarra (victor): If window.print is called to quickly, the browser
  // won't have time to set up the event listener that updates the document title
  setTimeout(window.print, 500);

  window.onfocus = () => {
    window.close();
  };
});

useHead({
  htmlAttrs: {
    style: "color-scheme: light;",
    class: "light",
  },
});
</script>
