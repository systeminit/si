<template>
  <div
    v-if="
      secretStore.secretAgent && data && data.items && data.items.length > 0
    "
    class="relative bg-neutral-800 overflow-hidden w-full"
  >
    <NextHeader :hero-words="false" />
    <div
      v-for="blog in data.items"
      :key="blog.id"
      class="relative px-4 sm:px-6 lg:px-8 bg-neutral-800"
    >
      <div
        class="text-lg max-w-prose mx-auto pt-10 bg-neutral-100 rounded-lg p-10 mb-10"
      >
        <h1>
          <span
            class="block text-3xl text-center leading-8 font-extrabold tracking-tight text-neutral-800 sm:text-4xl"
          >
            {{ blog.fields.title }}
          </span>
        </h1>
        <div class="mt-6 prose prose-gray prose-lg text-gray-100 mx-auto">
          <figure v-if="blog.fields.heroImage">
            <img
              class="w-full rounded-lg"
              :src="assetUrl(blog.fields.heroImage.fields.file.url)"
              alt=""
              width="1310"
              height="873"
            />
          </figure>
        </div>

        <div
          class="prose prose-neutral dark:prose-invert"
          v-html="blogAsMarkdown"
        ></div>
      </div>
    </div>
    <div class="bg-neutral-700">
      <div
        class="max-w-7xl mx-auto text-center py-12 px-4 sm:px-6 lg:py-16 lg:px-8"
      >
        <h2
          class="text-3xl font-extrabold tracking-tight text-gray-100 sm:text-4xl"
        >
          <span class="block">Want to talk about DevOps?</span>
          <span class="block">Come join us in Discord.</span>
        </h2>
        <div class="mt-8 flex justify-center">
          <iframe
            src="https://discord.com/widget?id=955539345538957342&theme=dark"
            width="350"
            height="500"
            allowtransparency="true"
            frameborder="0"
            sandbox="allow-popups allow-popups-to-escape-sandbox allow-same-origin allow-scripts"
          ></iframe>
        </div>
      </div>
    </div>
    <NextFooter />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
const secretStore = useSecretStore();
const config = useRuntimeConfig();
const contentful = await useContentful(config);
const marked = useMarked();
const route = useRoute();
const { data } = useAsyncData("pages", () => {
  return contentful.getEntries({
    content_type: "blogPost",
    "fields.slug": route.params.slug,
    order: "-sys.createdAt",
  });
});
const assetUrl = (url) => {
  return `https://${url}`;
};
const blogAsMarkdown = computed(() => {
  if (data.value.items) {
    const blog = data.value.items[0];
    return marked(blog.fields.body);
  } else {
    return "";
  }
});
</script>
