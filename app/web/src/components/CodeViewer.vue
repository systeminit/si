<template>
  <div class="flex flex-col w-full max-h-full overflow-hidden">
    <div
      class="flex flex-row items-center justify-between p-2 text-base align-middle"
      :class="titleClasses"
    >
      <!-- NOTE(nick): add defaults for title if the need arises -->
      <slot name="title"></slot>

      <div class="flex">
        <SiButtonIcon
          tooltipText="Copy code to clipboard"
          ignoreTextColor
          icon="clipboard-copy"
          @click="copyCodeToClipboard"
        />

        <slot name="actionButtons"></slot>
      </div>
    </div>
    <div
      :class="
        clsx(
          'w-full h-full overflow-auto',
          border && 'border',
          themeClasses('border-neutral-300', 'dark:border-neutral-600'),
        )
      "
      class="mt-4"
    >
      <div
        ref="editorMountRef"
        class="w-full h-full overflow-auto"
        @keyup.stop
        @keydown.stop
      ></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import { ref, computed, watch, PropType, onMounted, onBeforeMount } from "vue";
import { basicSetup, EditorView } from "codemirror";
import { StreamLanguage } from "@codemirror/language";

import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { gruvboxDark } from "cm6-theme-gruvbox-dark";
import { basicLight } from "cm6-theme-basic-light";
import {
  EditorState,
  Compartment,
  Extension,
  StateEffect,
} from "@codemirror/state";

import { properties as JsonModeParser } from "@codemirror/legacy-modes/mode/properties";
import { yaml as YamlModeParser } from "@codemirror/legacy-modes/mode/yaml";
import { diff as DiffModeParser } from "@codemirror/legacy-modes/mode/diff";
import clsx from "clsx";
import { themeClasses, useTheme } from "@si/vue-lib/design-system";
import { CodeLanguage } from "@/api/sdf/dal/code_view";

import SiButtonIcon from "@/components/SiButtonIcon.vue";

const props = defineProps({
  code: { type: String },
  codeLanguage: { type: String as PropType<CodeLanguage>, default: "unknown" },

  // // could add validation fns?
  // // Format: "0.0px"
  fontSize: { type: String },
  // // Format: "0.0px" or "0%"
  height: { type: String },
  titleClasses: { type: String, default: "h-10" },
  border: { type: Boolean, default: false },
});

const { theme } = useTheme();

const editorMountRef = ref();
const readOnly = new Compartment();
let editorView: EditorView | undefined;

// any new languages we want to support need to be added here
const CODE_PARSER_LOOKUP = {
  diff: DiffModeParser,
  json: JsonModeParser,
  yaml: YamlModeParser,
  // TODO: what do we want to do here...?
  unknown: YamlModeParser,
};

const editorThemeExtension = computed(() => {
  return {
    dark: gruvboxDark,
    light: basicLight,
  }[theme.value];
});

const editorStyleExtension = computed(() => {
  const activeLineHighlight = theme.value === "dark" ? "#7c6f64" : "#e0dee9";
  return EditorView.theme({
    "&": {
      height: "100%",
      ..._.pick(props, "fontSize", "height"),
    },
    ".cm-scroller": { overflow: "auto" },
    ".cm-focused .cm-selectionBackground .cm-activeLine, .cm-selectionBackground, .cm-content .cm-activeLine ::selection":
      { backgroundColor: `${activeLineHighlight} !important` },
  });
});

const editorExtensionList = computed<Extension[]>(() => {
  return [
    basicSetup,
    editorThemeExtension.value,
    editorStyleExtension.value,
    keymap.of([indentWithTab]),
    StreamLanguage.define(CODE_PARSER_LOOKUP[props.codeLanguage]),
    readOnly.of(EditorState.readOnly.of(true)),
    EditorView.lineWrapping,
  ];
});

function initCodeMirrorEditor() {
  editorView = new EditorView({
    state: EditorState.create({
      doc: props.code,
      extensions: editorExtensionList.value,
    }),
    parent: editorMountRef.value,
  });
}
function teardownCodeMirrorEditor() {
  editorView?.destroy();
}

function syncEditorConfig() {
  editorView?.dispatch({
    effects: StateEffect.reconfigure.of(editorExtensionList.value),
  });
}

function syncEditorCode() {
  if (!editorView) return;
  editorView.dispatch({
    changes: {
      from: 0,
      to: editorView.state.doc.length,
      insert: props.code,
    },
    effects: readOnly.reconfigure(EditorState.readOnly.of(true)),
  });
}

onMounted(initCodeMirrorEditor);
onBeforeMount(teardownCodeMirrorEditor);
watch(editorExtensionList, syncEditorConfig, { immediate: true });
watch(() => props.code, syncEditorCode);

// This doesn't work on IE, do we care? (is it polyfilled by our build system?)
// RE ^^: https://www.youtube.com/watch?v=Ram7AKbtkGE
function copyCodeToClipboard() {
  if (!editorView) return;
  const code = editorView.state.doc.toString().trim();
  navigator.clipboard.writeText(code);
}
</script>
