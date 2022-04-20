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
        <h3>
          <span class="block text-xl text-center leading-8 tracking-tight text-neutral-600 sm:text-md">
            By {{ blog.fields.author.fields.name }}
          </span>
        </h3>
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
          class="prose prose-neutral prose-invert"
          v-html="blogAsMarkdown"
        ></div>
        <div class="flex bg-neutral-600 rounded-lg mt-5 mb-5">
          <div class="ml-5 flex-shrink-0 self-center">
            <img
              :src="assetUrl(blog.fields.author.fields.image.fields.file.url)"
              class="h-16 w-16 rounded-full"
            />
          </div>
          <div class="m-5">
            <h4 class="prose font-bold prose-invert">
              {{ blog.fields.author.fields.name }},
              {{ blog.fields.author.fields.title }}
            </h4>
            <p class="mt-1 prose prose-invert">
              {{ blog.fields.author.fields.shortBio }}
            </p>
          </div>
        </div>
      </div>
    </div>
    <div class="bg-neutral-700">
      <div
        class="max-w-7xl mx-auto text-center py-12 px-4 sm:px-6 lg:py-16 lg:px-8"
      >
        <h2
          class="text-3xl font-extrabold tracking-tight text-gray-100 sm:text-4xl"
        >
          <span class="block">Have a DevOps papercut?</span>
          <span class="block">Tell us about it in Discord.</span>
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
