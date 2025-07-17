<template>
  <div class="w-full h-full ph-no-capture">
    <div v-if="!noVim" class="absolute right-xs top-xs flex gap-xs">
      <VButton
        v-if="disabled"
        class="pointer-events-none"
        icon="read-only"
        size="xs"
        tone="warning"
        variant="ghost"
      >
        Read-only
      </VButton>
      <template v-else>
        <VButton
          v-if="canFormat"
          label="Format"
          size="xs"
          tone="neutral"
          @click="format"
        />
        <VButton
          v-tooltip="vimEnabled ? 'Disable Vim Mode' : 'Enable Vim Mode'"
          :tone="vimEnabled ? 'success' : 'neutral'"
          icon="logo-vim"
          size="xs"
          @click="vimEnabled = !vimEnabled"
        />
      </template>
    </div>
    <div ref="editorMount" class="h-full" @keyup.stop @keydown.stop />
  </div>
</template>

<script lang="ts" setup>
import { onBeforeUnmount, computed, ref, watch } from "vue";
import * as _ from "lodash-es";
import { basicSetup, EditorView } from "codemirror";
import { Compartment, EditorState, StateEffect } from "@codemirror/state";
import {
  ViewUpdate,
  keymap,
  hoverTooltip,
  Tooltip,
  showTooltip,
  getTooltip,
} from "@codemirror/view";
import { indentWithTab, historyField } from "@codemirror/commands";
import { githubLight } from "@fsegurai/codemirror-theme-github-light";
import { githubDark } from "@fsegurai/codemirror-theme-github-dark";
import { javascript as CodemirrorJsLang } from "@codemirror/lang-javascript";
import { json as CodemirrorJsonLang } from "@codemirror/lang-json";
import { yaml as CodemirrorYamlLang } from "@codemirror/lang-yaml";
import { linter, lintGutter } from "@codemirror/lint";
import { useTheme, VButton } from "@si/vue-lib/design-system";
import { vim, Vim } from "@replit/codemirror-vim";
import storage from "local-storage-fallback";
import beautify from "js-beautify";
import * as Y from "yjs";
import { WebsocketProvider } from "y-websocket";
import { yCollab, yUndoManagerKeymap } from "yjs-codemirror-plugin";
import { useAuthStore } from "@/store/auth.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import {
  createTypescriptSource,
  GetTooltipFromPos,
} from "@/utils/typescriptLinter";

const props = defineProps({
  id: String,
  modelValue: { type: String, required: true },
  recordId: { type: String, required: true },
  disabled: { type: Boolean },
  json: Boolean,
  typescript: { type: String },
  yaml: Boolean,
  noLint: Boolean,
  noVim: Boolean,
  debounceUpdate: { type: Boolean, default: false },
});

const emit = defineEmits<{
  "update:modelValue": [v: string];
  blur: [v: string];
  change: [id: string, v: string, debounce: boolean];
  close: [];
}>();

const changeSetsStore = useChangeSetsStore();
const authStore = useAuthStore();

const editorMount = ref(); // div (template ref) where we will mount the editor
let view: EditorView; // instance of the CodeMirror editor

function getCursorInfo(state: EditorState) {
  const selection = state.selection.main;
  const cursorPosition = selection.head;

  const line = state.doc.lineAt(cursorPosition);
  const lineNumber = line.number;

  const column = cursorPosition - line.from;

  return {
    lineNumber,
    column,
    overallPosition: cursorPosition,
  };
}

function setCursorPosition(
  view: EditorView,
  lineNumber: number,
  column: number,
) {
  const line = view.state.doc.line(lineNumber);
  const position = Math.min(line.from + column, line.to);

  const transaction = view.state.update({
    selection: { anchor: position },
    scrollIntoView: true,
  });

  view.dispatch(transaction);
}

const canFormat = computed(() => props.json || props.typescript || props.yaml);

const format = (): boolean => {
  if (props.disabled || !yText) return false;

  if (canFormat.value) {
    const preFormatPosition = getCursorInfo(view.state);
    const text = beautify(view.state.doc.toString());
    if (text !== view.state.doc.toString()) {
      yText.delete(0, yText.length);
      yText.insert(0, text);
      setCursorPosition(
        view,
        preFormatPosition.lineNumber,
        preFormatPosition.column,
      );
    }
  }
  return true;
};

const localStorageHistoryBufferKey = computed(
  () => `code-mirror-state-${changeSetsStore.selectedChangeSetId}-${props.id}`,
);

function onEditorValueUpdated(update: ViewUpdate) {
  if (!update.docChanged) return;

  emit("update:modelValue", update.state.doc.toString());
  emit("change", props.recordId, view.state.doc.toString(), true);

  const serializedState = update.view.state.toJSON({ history: historyField });
  if (serializedState.history) {
    serializedState.history.done.splice(
      0,
      Math.max(serializedState.history.done.length - 50, 0),
    );
    serializedState.history.undone.splice(
      0,
      Math.max(serializedState.history.undone.length - 50, 0),
    );
    window.localStorage.setItem(
      localStorageHistoryBufferKey.value,
      JSON.stringify({
        history: serializedState.history,
        timestamp: new Date(),
      }),
    );
  }
}

// set up all compartments
const language = new Compartment();
const readOnly = new Compartment();
const themeCompartment = new Compartment();
const lintCompartment = new Compartment();
const autocompleteCompartment = new Compartment();
const styleExtensionCompartment = new Compartment();
const vimCompartment = new Compartment();
const hoverTooltipCompartment = new Compartment();
const removeTooltipOnUpdateCompartment = new Compartment();
const yCompartment = new Compartment();

// Theme / style ///////////////////////////////////////////////////////////////////////////////////////////
const { theme: appTheme } = useTheme();
const codeMirrorTheme = computed(() =>
  appTheme.value === "dark" ? githubDark : githubLight,
);

const styleExtension = computed(() => {
  const activeLineHighlight = appTheme.value === "dark" ? "#2d333b" : "#f6f8fa";
  const tooltipBackground = appTheme.value === "dark" ? "#161b22" : "#ffffff";
  const tooltipBorder = appTheme.value === "dark" ? "#30363d" : "#d0d7de";
  const tooltipTagText = appTheme.value === "dark" ? "#58a6ff" : "#0969da";

  return EditorView.theme({
    "&": { height: "100%" },
    ".cm-scroller": { overflow: "auto" },
    // Vim style: https://github.com/replit/codemirror-vim/blob/d7d9ec2ab438571f500dfd21b37da733fdba47fe/src/index.ts#L25-L42
    ".cm-vim-panel, .cm-vim-panel input": {
      padding: "0px 10px",
      fontSize: "14px",
      minHeight: "0em",
    },
    ".cm-focused .cm-selectionBackground .cm-activeLine, .cm-selectionBackground, .cm-content .cm-activeLine ::selection":
      { backgroundColor: `${activeLineHighlight} !important` },
    ".cm-tooltip-autocomplete": {
      backgroundColor: `${tooltipBackground} !important`,
      border: `1px solid ${tooltipBorder} !important`,
      borderRadius: "0.25rem",
    },
    ".cm-tooltip-lint": {
      backgroundColor: `${tooltipBackground} !important`,
      border: `1px solid ${tooltipBorder} !important`,
      borderRadius: "0 0.25rem 0.25rem 0",
    },
    ".cm-tooltip": {
      backgroundColor: `${tooltipBackground} !important`,
      border: `1px solid ${tooltipBorder} !important`,
      borderRadius: "0.25rem",
      padding: ".5rem !important",
      whiteSpace: "pre-wrap",
      fontFamily: "monospace",
      maxWidth: "60vw",
      maxHeight: "300px",
      overflowY: "auto",
      lineHeight: "1.5",
    },
    ".cm-tooltip-doc-signature": {
      paddingBottom: ".5rem",
      fontWeight: "bold",
    },
    ".cm-tooltip-doc-details": {
      paddingBottom: ".5rem",
      fontStyle: "italic",
    },
    ".cm-tooltip-doc-tag": {},
    ".cm-tooltip-doc-tag-name": {
      fontWeight: "bold",
      color: `${tooltipTagText}`,
    },
    ".cm-tooltip-doc-tag-info": {},
    ".cm-tooltip-doc-tag-example": {
      fontStyle: "italic",
    },
  });
});
watch(codeMirrorTheme, () => {
  view.dispatch({
    effects: [
      themeCompartment.reconfigure(codeMirrorTheme.value),
      styleExtensionCompartment.reconfigure(styleExtension.value),
    ],
  });
});

// VIM MODE ////////////////////////////////////////////////////////////////////////////////////////
const VIM_MODE_STORAGE_KEY = "SI:VIM_MODE";
const vimEnabled = ref(
  !props.noVim && storage.getItem(VIM_MODE_STORAGE_KEY) === "true",
);
watch(vimEnabled, (useVim) => {
  storage.setItem(VIM_MODE_STORAGE_KEY, useVim ? "true" : "false");
  view.dispatch({
    effects: [vimCompartment.reconfigure(useVim ? vim({ status: true }) : [])],
  });
});
// Emit when the user writes (i.e. ":w") in vim mode.
Vim.defineEx("write", "w", format);
// Emit when the user quits in vim mode.
Vim.defineEx("quit", "q", onVimExit);

// Code Tooltip /////////////////////////////////////////////////////////////////////////////////

const codeTooltip = {
  currentTooltip: null as Tooltip | null,
  destroy() {
    if (this.currentTooltip) {
      const tt = getTooltip(view, this.currentTooltip);
      if (tt?.destroy) tt?.destroy();
      this.currentTooltip = null;
    }
  },
  update() {
    this.currentTooltip = GetTooltipFromPos(view.state.selection.main.head);
    view.dispatch({
      effects: [
        StateEffect.appendConfig.of(showTooltip.of(this.currentTooltip)),
      ],
    });
  },
  toggle() {
    if (codeTooltip.currentTooltip) {
      codeTooltip.destroy();
      return true;
    }
    codeTooltip.update();
    return true;
  },
};

let wsProvider: WebsocketProvider | undefined;
let yText: Y.Text | undefined;
onBeforeUnmount(() => {
  wsProvider?.destroy();
});

// Initialization /////////////////////////////////////////////////////////////////////////////////
const mountEditor = async () => {
  if (!editorMount.value) return;
  const extensions = [basicSetup];

  if (props.typescript) {
    if (!props.noLint) {
      const {
        lintSource,
        autocomplete,
        hoverTooltipSource,
        removeTooltipOnUpdateSource,
      } = await createTypescriptSource(props.typescript);

      extensions.push(autocompleteCompartment.of(autocomplete));
      extensions.push(lintCompartment.of(linter(lintSource)));
      extensions.push(
        hoverTooltipCompartment.of(hoverTooltip(hoverTooltipSource)),
      );
      extensions.push(
        removeTooltipOnUpdateCompartment.of(
          removeTooltipOnUpdateSource(codeTooltip),
        ),
      );
      extensions.push(lintGutter());
    }
    extensions.push(language.of(CodemirrorJsLang()));
  }

  if (props.json) {
    extensions.push(language.of(CodemirrorJsonLang()));
  }

  if (props.yaml) {
    extensions.push(language.of(CodemirrorYamlLang()));
  }

  const ydoc = new Y.Doc();
  yText = ydoc.getText("codemirror");

  const finishEditor = () => {
    const config = {
      doc: yText?.toString() ?? "",
      extensions: extensions.concat([
        themeCompartment.of(codeMirrorTheme.value),
        styleExtensionCompartment.of(styleExtension.value),
        keymap.of([
          indentWithTab,
          { key: "ctrl-m", run: codeTooltip?.toggle },
          { key: "cmd-m", run: codeTooltip?.toggle },
          { key: "ctrl-s", run: format },
          { key: "cmd-s", run: format },
        ]),

        readOnly.of(EditorState.readOnly.of(props.disabled)),
        vimCompartment.of(vimEnabled.value ? vim({ status: true }) : []),
        EditorView.updateListener.of(onEditorValueUpdated),
        EditorView.lineWrapping,
      ]),
    };

    let editorState;
    const state = null; /* props.id
      ? window.localStorage.getItem(localStorageHistoryBufferKey.value)
      : null; */
    if (state) {
      editorState = EditorState.fromJSON(
        {
          doc: config.doc,
          selection: { ranges: [{ anchor: 0, head: 0 }], main: 0 },
          history: JSON.parse(state).history,
        },
        config,
        { history: historyField },
      );
    } else {
      editorState = EditorState.create(config);
    }

    view?.destroy();
    view = new EditorView({
      state: editorState,
      parent: editorMount.value,
    });

    // we don't need to fire anything on blur because the updates are debounced as users type
  };

  extensions.push(keymap.of([...yUndoManagerKeymap]));

  // const undoManager = new Y.UndoManager(yText);
  extensions.push(
    yCompartment.of(yCollab(yText, null)), // , { undoManager })),
  );

  yText.delete(0, yText.length);
  yText.insert(0, props.modelValue);
  finishEditor();

  for (const key in window.localStorage) {
    if (key.startsWith("code-mirror-state-")) {
      const json = window.localStorage.getItem(key);
      if (!json) continue;
      const obj = JSON.parse(json);
      const millisSince =
        new Date().getTime() - new Date(obj.timestamp).getTime();
      const weekInMillis = 7 * 24 * 60 * 1000;
      if (millisSince > weekInMillis) {
        window.localStorage.removeItem(key);
      }
    }
  }
};

// always up the code editor with the new text that comes from the prop
// Note: props are read only, and will change when the selected func/variant changes
// this does not change as a result of a user typing
const watching = computed(() => [
  () => props.id,
  () => props.typescript,
  () => props.disabled,
  () => props.json,
  () => props.yaml,
  () => props.noLint,
  () => authStore.user?.name,
  () => (props.disabled ? props.modelValue : null),
  editorMount,
]);
watch(watching.value, mountEditor);

function onVimExit() {
  emit("close");
  return true; // codemirror needs this when used as a "command"
}

// This is only used for passing in multiplayer updates which otherwise don't come through modelValue properly
const forceUpdateContent = (value = props.modelValue) => {
  if (!view) return;

  const selection = view.state.selection;
  view.dispatch({
    changes: {
      from: 0,
      to: view.state.doc.length,
      insert: value,
    },
    selection,
  });
};

defineExpose({
  forceUpdateContent,
});
</script>

<style>
.cm-editor .cm-content {
  font-size: 14px;
}

.cm-editor .cm-gutter {
  font-size: 14px;
}

.cm-editor .cm-ySelectionInfo {
  position: absolute;
  color: black;
  font-size: 0.75rem;
  font-weight: 600;
  font-family: monospace;
}

.cm-editor > .cm-scroller {
  overflow-x: hidden;
}
</style>
