<template>
  <div class="flex flex-col w-full max-h-full overflow-hidden">
    <div
      class="flex flex-row items-center justify-between h-10 py-2 text-base align-middle"
    >
      <!-- NOTE(nick): add defaults for title if the need arises -->
      <slot name="title"></slot>

      <div class="flex">
        <SiButtonIcon
          tooltip-text="Copy code to clipboard"
          ignore-text-color
          @click="copyCodeToClipboard"
        >
          <ClipboardCopyIcon />
        </SiButtonIcon>

        <slot name="actionButtons"></slot>
      </div>
    </div>
    <div class="w-full h-full overflow-auto">
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
import _ from "lodash";
import { ref, computed, watch, PropType, onMounted, onBeforeMount } from "vue";
import { EditorState, EditorView, basicSetup } from "@codemirror/basic-setup";
import { StreamLanguage } from "@codemirror/stream-parser";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { gruvboxDark } from "cm6-theme-gruvbox-dark";
import { basicLight } from "cm6-theme-basic-light";
import { Compartment, Extension, StateEffect } from "@codemirror/state";

import { properties as JsonModeParser } from "@codemirror/legacy-modes/mode/properties";
import { yaml as YamlModeParser } from "@codemirror/legacy-modes/mode/yaml";
import { diff as DiffModeParser } from "@codemirror/legacy-modes/mode/diff";
import { ClipboardCopyIcon } from "@heroicons/vue/solid";
import { CodeLanguage } from "@/api/sdf/dal/code_view";
// NOTE(nick): this took a long ass time to find. Javascript's JSON mode doesn't work. This does.

import { useTheme } from "@/composables/injectTheme";
import { ThemeValue } from "@/observable/theme";

import SiButtonIcon from "@/atoms/SiButtonIcon.vue";

const props = defineProps({
  code: { type: String },
  codeLanguage: { type: String as PropType<CodeLanguage>, default: "unknown" },

  // // could add validation fns?
  // // Format: "0.0px"
  fontSize: { type: String },
  // // Format: "0.0px" or "0%"
  height: { type: String },
  forceTheme: { type: String as PropType<ThemeValue> },
});

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

const theme = useTheme();

const editorThemeExtension = computed(() => {
  return {
    dark: gruvboxDark,
    light: basicLight,
  }[props.forceTheme || theme.value];
});

const editorStyleExtension = computed(() => {
  return EditorView.theme({
    "&": {
      height: "100%",
      ..._.pick(props, "fontSize", "height"),
    },
    ".cm-scroller": { overflow: "auto" },
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
