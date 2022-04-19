<template>
  <div class="bg-neutral-800 w-full" v-if="secretStore.secretAgent">
    <div class="relative overflow-hidden">
      <NextHeader :hero-words="false" />
      <div class="relative bg-neutral-800 pb-20 px-4 sm:px-6 lg:pb-28 lg:px-8">
        <div class="absolute inset-0">
          <div class="bg-neutral-800 h-1/3 sm:h-2/3" />
        </div>
        <div class="relative max-w-7xl mx-auto">
          <div class="text-center">
            <h2
              class="text-3xl tracking-tight font-extrabold text-gray-100 sm:text-4xl"
            >
              Blog
            </h2>
            <p class="mt-3 max-w-2xl mx-auto text-xl text-gray-200 sm:mt-4">
              DevOps, Engineering, and System Initiative.
            </p>
          </div>
          <div
            class="mt-12 max-w-lg mx-auto grid gap-5 lg:grid-cols-3 lg:max-w-none"
          >
            <div
              v-for="post in blogPosts"
              :key="post.fields.title"
              class="flex flex-col rounded-lg shadow-lg overflow-hidden"
            >
              <div class="flex-shrink-0">
                <img
                  class="h-48 w-full object-cover"
                  :src="assetUrl(post.fields.heroImage.fields.file.url)"
                  alt=""
                />
              </div>
              <div class="flex-1 bg-white p-6 flex flex-col justify-between">
                <div class="flex-1">
                  <a :href="blogUrl(post.fields.slug)" class="block mt-2">
                    <p class="text-xl font-semibold text-gray-900">
                      {{ post.fields.title }}
                    </p>
                    <p class="mt-3 text-base text-gray-500">
                      {{ post.fields.description }}
                    </p>
                  </a>
                </div>
                <div class="mt-6 flex items-center">
                  <div class="flex-shrink-0">
                    <span class="sr-only">{{
                      post.fields.author.fields.name
                    }}</span>
                    <img
                      class="h-10 w-10 rounded-full"
                      :src="
                        assetUrl(
                          post.fields.author.fields.image.fields.file.url,
                        )
                      "
                      alt=""
                    />
                  </div>
                  <div class="ml-3">
                    <p class="text-sm font-medium text-gray-900">
                      {{ post.fields.author.fields.name }}
                    </p>
                    <div class="flex space-x-1 text-sm text-gray-500">
                      <time :datetime="post.fields.publishDate">
                        {{ post.fields.publishDate }}
                      </time>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
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
const { data } = await useAsyncData("pages", () => {
  return contentful.getEntries({
    content_type: "blogPost",
    order: "-sys.createdAt",
  });
});

const blogPosts = computed(() => {
  if (data.value) {
    return data.value.items;
  } else {
    return [];
  }
});

const assetUrl = (url: string) => {
  return `https://${url}`;
};

const blogUrl = (slug: string) => {
  return `/blog-${slug}`;
};
</script>
