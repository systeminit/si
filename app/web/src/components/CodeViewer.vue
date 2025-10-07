<template>
  <div
    ref="mainDivRef"
    :class="
      clsx(
        'flex flex-col w-full overflow-hidden relative',
        !disableScroll && 'max-h-full',
      )
    "
  >
    <div
      v-if="showTitle"
      class="flex flex-row items-center justify-between p-xs text-base align-middle"
      :class="titleClasses"
    >
      <slot name="title">
        <TruncateWithTooltip v-if="title">{{ title }}</TruncateWithTooltip>
      </slot>

      <div class="flex">
        <IconButton
          v-if="allowCopy"
          :tooltip="copyTooltip"
          icon="clipboard-copy"
          iconTone="shade"
          tooltipPlacement="top"
          @click="copyCodeToClipboard"
        />
        <slot name="actionButtons"></slot>
      </div>
    </div>
    <IconButton
      v-if="!showTitle && allowCopy"
      :tooltip="copyTooltip"
      icon="clipboard-copy"
      iconTone="shade"
      tooltipPlacement="top"
      :class="
        clsx(
          'absolute z-10 right-xs',
          mainDivTallEnoughForCopyIconPadding ? 'top-xs' : 'top-0',
        )
      "
      @click="copyCodeToClipboard"
    />
    <div
      v-if="numberOfLinesInCode > 1 || forceLineNumbers"
      :class="
        clsx(
          'w-full h-full overflow-auto scrollable',
          border && 'border',
          themeClasses('border-neutral-300', 'border-neutral-600'),
        )
      "
    >
      <div
        ref="editorMountRef"
        class="w-full h-full overflow-auto scrollable"
        @keyup.stop
        @keydown.stop
      ></div>
    </div>
    <div
      v-else
      :class="
        clsx(
          'w-full h-full',
          border && 'border',
          'flex font-mono break-all text-wrap overflow-hidden p-2xs rounded',
          themeClasses(
            'bg-neutral-100 border-neutral-300',
            'bg-neutral-800 border-neutral-600',
          ),
        )
      "
    >
      <div class="overflow-auto scrollable">{{ code }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import {
  ref,
  computed,
  watch,
  PropType,
  onMounted,
  onBeforeMount,
  nextTick,
} from "vue";
import { basicSetup, EditorView } from "codemirror";
import { StreamLanguage } from "@codemirror/language";

import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { githubLight } from "@fsegurai/codemirror-theme-github-light";
import { githubDark } from "@fsegurai/codemirror-theme-github-dark";
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
import {
  IconButton,
  themeClasses,
  TruncateWithTooltip,
  useTheme,
} from "@si/vue-lib/design-system";
import { javascript as CodemirrorJsLang } from "@codemirror/lang-javascript";
import { CodeLanguage } from "@/api/sdf/dal/code_view";

const props = defineProps({
  code: { type: String },
  codeLanguage: { type: String as PropType<CodeLanguage>, default: "unknown" },

  // // could add validation fns?
  // // Format: "0.0px"
  fontSize: { type: String, default: "13px" },
  // // Format: "0.0px" or "0%"
  height: { type: String },
  showTitle: { type: Boolean },
  allowCopy: { type: Boolean, default: true },
  title: { type: String },
  titleClasses: { type: String, default: "h-10 text-lg" },
  border: { type: Boolean, default: false },
  disableScroll: { type: Boolean },
  copyTooltip: { type: String, default: "Copy code to clipboard" },
  forceLineNumbers: { type: Boolean }, // forces line numbers even for a one line string
});

const numberOfLinesInCode = computed(() => {
  return (String(props.code).match(/\n/g) || "").length + 1;
});

const mainDivRef = ref<HTMLElement>();

const mainDivTallEnoughForCopyIconPadding = computed(
  () =>
    mainDivRef.value && mainDivRef.value?.getBoundingClientRect().height > 32,
);

const { theme } = useTheme();

const editorMountRef = ref();
const readOnly = new Compartment();
let editorView: EditorView | undefined;

// any new languages we want to support need to be added here
const CODE_PARSER_LOOKUP = {
  diff: DiffModeParser,
  json: JsonModeParser,
  yaml: YamlModeParser,
  string: YamlModeParser,
  // TODO: what do we want to do here...?
  unknown: YamlModeParser,
};

const editorThemeExtension = computed(() => {
  return {
    dark: githubDark,
    light: githubLight,
  }[theme.value];
});

const editorStyleExtension = computed(() => {
  const activeLineHighlight = theme.value === "dark" ? "#2d333b" : "#f6f8fa";
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

const javascriptLang = new Compartment();

const editorExtensionList = computed<Extension[]>(() => {
  const codeParser =
    props.codeLanguage === "javascript"
      ? javascriptLang.of(CodemirrorJsLang())
      : StreamLanguage.define(CODE_PARSER_LOOKUP[props.codeLanguage]);
  return [
    basicSetup,
    editorThemeExtension.value,
    editorStyleExtension.value,
    keymap.of([indentWithTab]),
    codeParser,
    readOnly.of(EditorState.readOnly.of(true)),
    EditorView.lineWrapping,
  ];
});

function initCodeMirrorEditor() {
  editorView = new EditorView({
    state: EditorState.create({
      doc:
        numberOfLinesInCode.value === 1 && props.forceLineNumbers
          ? `${props.code}\n`
          : props.code,
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
  // this is what `CodeEditor` does, on any change it remounts the editor
  teardownCodeMirrorEditor();
  initCodeMirrorEditor();
}

onMounted(initCodeMirrorEditor);
onBeforeMount(teardownCodeMirrorEditor);
watch(editorExtensionList, syncEditorConfig, { immediate: true });
watch(
  () => props.code,
  () => {
    nextTick(() => {
      syncEditorCode();
    });
  },
);

// This doesn't work on IE, do we care? (is it polyfilled by our build system?)
// RE ^^: https://www.youtube.com/watch?v=Ram7AKbtkGE
function copyCodeToClipboard() {
  if (numberOfLinesInCode.value < 2) {
    navigator.clipboard.writeText(props.code as string);
    return;
  }
  if (!editorView) return;
  const code = editorView.state.doc.toString().trim();
  navigator.clipboard.writeText(code);
}
</script>
