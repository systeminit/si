<template>
  <div ref="containerRef" class="richtext">
    <slot />
  </div>
</template>

<!-- having this helps with importing / IDE click through -->
<script lang="ts" setup>
import { onMounted, onUpdated, ref } from "vue";
import { useHead } from "@vueuse/head";

import hljs from "highlight.js/lib/core";
import hljsJsLang from "highlight.js/lib/languages/javascript";

/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-ignore
import hljsThemeLight from "highlight.js/styles/github.css?raw";
// @ts-ignore
import hljsThemeDark from "highlight.js/styles/github-dark.css?raw";
import { useTheme } from "../utils/theme_tools";

hljs.registerLanguage("javascript", hljsJsLang);
hljs.registerAliases("js", { languageName: "javascript" });

const hasCodeBlocks = ref(false);

const containerRef = ref<HTMLDivElement>();

// fairly naive way of doing this... we look for code blocks and replace the code with the highlighted html version
function highlightCode() {
  if (!containerRef.value) return;
  const codeEls = containerRef.value.querySelectorAll("code");
  if (codeEls.length) hasCodeBlocks.value = true;
  codeEls.forEach((codeEl) => {
    const code = codeEl.textContent;
    // class added like "language-xyz", we'll default to js if nothing set
    let language: string | undefined;
    codeEl.classList.forEach((c) => {
      if (c.startsWith("language-")) {
        language = c.replace("language-", "");
      }
    });
    if (!language) {
      language = "js";
      codeEl.classList.add(`language-${language}`);
    }

    const highlightedCode = hljs.highlight(code || "", { language });
    codeEl.classList.add("hljs");
    codeEl.innerHTML = highlightedCode.value;
  });
}

// dynamically add theme in a way we can toggle it
const { theme } = useTheme();
useHead(() => ({
  ...(hasCodeBlocks.value && {
    style: [
      {
        innerHTML: theme.value === "light" ? hljsThemeLight : hljsThemeDark,
        key: "hljs-theme-css",
      },
    ],
  }),
}));

onMounted(highlightCode);
onUpdated(highlightCode);
</script>

<style lang="less">
.richtext {
  // this styling can be a bit problematic when nesting actual components inside
  // so likely will need some work if we do that more...
  // for now, I try to not apply styling if something is in an element with class "escape"

  max-width: 100%;
  position: relative;

  line-height: 1.4em;

  > * {
    margin-bottom: 1em;
    &:last-child {
      margin-bottom: 0;
    }
  }

  :not(.escape) a:not(.vbutton) {
    html.dark & {
      color: @colors-action-300;
    }
    html.light & {
      color: @colors-action-500;
    }
    text-decoration: underline;
    text-decoration-thickness: 0.05em;
    text-underline-offset: 0.15em;

    &:hover {
      color: @colors-action-400;
    }
  }

  blockquote {
    padding-left: 2em;
  }

  > h1,
  > h2,
  > h3,
  > h4,
  > h5 {
    font-weight: bold;
    line-height: 1.4em;
    padding-top: 0.8em;
    &:first-child {
      padding-top: 0;
    }
  }
  > h1 {
    font-size: 24px;
  }
  > h2 {
    font-size: 20px;
  }
  > h3 {
    font-size: 18px;
  }
  > h4 {
    font-size: 16px;
    text-decoration: underline;
  }

  > ul {
    margin-bottom: 1em;
    padding-left: 1em;
    li {
      list-style-type: disc;

      // hide list bullet if icon
      &:has(> .icon:first-child) {
        list-style: none;
        margin-left: -1em;
      }
      > .icon:first-child {
        display: inline-block;
        vertical-align: bottom;
        margin-right: 0.5em;
      }
      li {
        list-style-type: circle;
        li {
          list-style-type: square;
        }
      }
      // display: flex;
      // align-items: center;
      padding-bottom: 0.3em;
    }
    ul {
      padding-left: 1em;
      padding-top: 0.5em;
    }
  }
  > img:not([width]),
  p > img:not([width]) {
    width: 100%;
    max-width: 700px;
    margin: 0 auto;
  }

  > table {
    width: 100%;

    th {
      background: rgba(0, 0, 0, 0.2);
    }

    tr,
    td,
    th {
      text-align: left;
      border: 1px solid white;
      html.light & {
        border-color: black;
      }
      padding: 0.5em;
      vertical-align: top;
    }
  }

  code {
    font-size: 13px;
    line-height: 1.3em;
  }
}
</style>
