<template>
  <div
    ref="divRef"
    :class="clsx('prose dark:prose-invert', disableMaxWidth && 'max-w-full')"
  >
    <VueMarkdown
      :source="props.source"
      :options="{ breaks: true, linkify: true, typographer: true }"
    />
  </div>
</template>

<script setup lang="ts">
import { tw } from "@si/vue-lib";
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { onMounted, ref } from "vue";
import VueMarkdown from "vue-markdown-render";

const props = defineProps<{
  source: string;
  removeMargins?: boolean;
  disableMaxWidth?: boolean;
}>();

const divRef = ref<HTMLDivElement>();

onMounted(() => {
  if (divRef.value && divRef.value.children[0]) {
    const root = divRef.value;

    const walkChildren = (el: Element) => {
      // Use this to add styles to particular elements in the Markdown
      if (el instanceof HTMLAnchorElement) {
        el.target = "_blank";
        el.classList.add(
          themeClasses(tw`text-action-500`, tw`text-action-300`),
          tw`no-underline`,
          tw`hover:underline`,
          tw`font-bold`,
        );
      }

      if (props.removeMargins) {
        el.classList.add(tw`m-0`);
      }

      Array.from(el.children).forEach((child) => walkChildren(child));
    };

    walkChildren(root);
  }
});
</script>
