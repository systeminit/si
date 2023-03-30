<template>
  <div v-if="docsLoaded" class="p-lg bg-white text-black">
    <RichText>
      <h1>{{ docs[urlDocSlug].title }}</h1>
      <Component :is="docs[urlDocSlug].component" />
    </RichText>
  </div>
</template>

<script setup lang="ts">
import { RichText } from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { ComponentOptions, nextTick, onBeforeMount, ref, watch } from "vue";
import { useRoute } from "vue-router";

const route = useRoute();
const _urlDocVersion = route.params.docVersion as string;
const urlDocSlug = route.params.docSlug as string;

// TODO: clean this up - currently copy/pasted from other legal page
const docsLoaded = ref(false);
const docs = {} as Record<
  string,
  {
    title: string;
    slug: string;
    fileName: string;
    component: ComponentOptions;
  }
>;
onBeforeMount(async () => {
  const docImports = import.meta.glob(`@/content/legal/2023-03-30/*.md`);
  for (const fileName in docImports) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const importedDoc = (await docImports[fileName]()) as any;
    const slug = fileName.replace(/.md$/, "").replace(/.*\/\d+-/, "");
    docs[slug] = {
      title: importedDoc.attributes.title,
      slug,
      fileName,
      component: importedDoc.VueComponent,
    };
  }
  docsLoaded.value = true;
});

watch(docsLoaded, () => {
  if (import.meta.env.SSR) return;
  if (docsLoaded.value) {
    // have to call nextTick so content gets rendered first
    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    nextTick(triggerPrint);
  }
});

function triggerPrint() {
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
