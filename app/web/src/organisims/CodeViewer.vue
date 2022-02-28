<template>
  <div v-if="props.componentId" class="flex flex-col w-full overflow-hidden">
    <div
      class="flex flex-row items-center justify-between h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">Component ID {{ props.componentId }} Code</div>
    </div>
    <div class="w-full h-full overflow-auto">
      <div
        ref="editorMount"
        class="w-full h-full"
        @keyup.stop
        @keydown.stop
      ></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { EditorState, EditorView, basicSetup } from "@codemirror/basic-setup";
import { yaml } from "@codemirror/legacy-modes/mode/yaml";
import { StreamLanguage } from "@codemirror/stream-parser";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { gruvboxDark } from "cm6-theme-gruvbox-dark";

const props = defineProps<{
  componentId: number;
}>();

const editorMount = ref(null);
onMounted(() => {
  if (editorMount.value) {
    const fixedHeightEditor = EditorView.theme({
      "&": { height: "100%" },
      ".cm-scroller": { overflow: "auto" },
    });

    const view = new EditorView({
      state: EditorState.create({
        doc: "---\nyaml: works\n",
        extensions: [
          basicSetup,
          gruvboxDark,
          fixedHeightEditor,
          keymap.of([indentWithTab]),
          StreamLanguage.define(yaml),
        ],
      }),
      parent: editorMount.value,
    });
  }
});
</script>

<style scoped>
.property-section-bg-color {
  background-color: #292c2d;
}
</style>
