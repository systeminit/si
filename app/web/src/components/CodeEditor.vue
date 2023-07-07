<template>
  <div class="w-full h-full ph-no-capture">
    <div v-if="!noVim" class="absolute right-xs top-xs flex gap-xs">
      <VButton
        v-if="disabled"
        size="xs"
        tone="warning"
        icon="read-only"
        variant="ghost"
        class="pointer-events-none"
      >
        Read-only
      </VButton>
      <VButton
        v-else
        size="xs"
        :tone="vimEnabled ? 'success' : 'neutral'"
        icon="logo-vim"
        @click="vimEnabled = !vimEnabled"
      />
    </div>
    <div ref="editorMount" class="h-full" @keyup.stop @keydown.stop />
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import * as _ from "lodash-es";
import { basicSetup, EditorView } from "codemirror";
import { Compartment, EditorState } from "@codemirror/state";
import { ViewUpdate, keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { gruvboxDark } from "cm6-theme-gruvbox-dark";
import { basicLight } from "cm6-theme-basic-light";
import { javascript as CodemirrorJsLang } from "@codemirror/lang-javascript";
import { json as CodemirrorJsonLang } from "@codemirror/lang-json";
import { linter, lintGutter } from "@codemirror/lint";
import { useTheme, VButton } from "@si/vue-lib/design-system";
import { vim, Vim } from "@replit/codemirror-vim";
import storage from "local-storage-fallback";
import { createTypescriptSource } from "@/utils/typescriptLinter";

const props = defineProps({
  modelValue: { type: String, required: true },
  disabled: { type: Boolean },
  json: Boolean,
  typescript: { type: String },
  noLint: Boolean,
  noVim: Boolean,
  debounceUpdate: { type: Boolean, default: true },
});

const emit = defineEmits<{
  "update:modelValue": [v: string];
  change: [v: string];
  explicitSave: [];
}>();

const editorMount = ref(); // div (template ref) where we will mount the editor
let view: EditorView; // instance of the CodeMirror editor

// our local copy of code
const draftValue = ref(props.modelValue || "");

// if v-model value changes, update our local draft
watch(
  () => props.modelValue,
  () => {
    if (draftValue.value !== props.modelValue) {
      draftValue.value = props.modelValue || "";
    }
  },
);

// when our draft value changes, make sure editor is in sync, and emit (debounced) update event
watch(
  () => draftValue.value,
  () => {
    const currentEditorValue = view?.state.doc.toString();

    // update the code in the code mirror instance value (if it's not matching)
    if (view && currentEditorValue !== draftValue.value) {
      view.update([
        view.state.update({
          changes: {
            from: 0,
            to: view.state.doc.length,
            insert: draftValue.value,
          },
        }),
      ]);
    }

    if (props.debounceUpdate) debouncedEmitUpdatedValue();
    else emitUpdatedValue();
  },
);

// when editor value changes, update our draft value
function onEditorValueUpdated(update: ViewUpdate) {
  if (!update.docChanged) return;
  console.log("editor value changed");
  const newEditorValue = update.view.state.doc.toString();
  if (newEditorValue !== draftValue.value) {
    draftValue.value = newEditorValue;
  }
}
function emitUpdatedValue() {
  console.log("emit new value");
  emit("update:modelValue", draftValue.value);
  emit("change", draftValue.value);
}
const debouncedEmitUpdatedValue = _.debounce(emitUpdatedValue, 3000);

// set up all compartments
const language = new Compartment();
const readOnly = new Compartment();
const themeCompartment = new Compartment();
const lintCompartment = new Compartment();
const autocompleteCompartment = new Compartment();
const styleExtensionCompartment = new Compartment();
const vimCompartment = new Compartment();

// Theme / style ///////////////////////////////////////////////////////////////////////////////////////////
const { theme: appTheme } = useTheme();
const codeMirrorTheme = computed(() =>
  appTheme.value === "dark" ? gruvboxDark : basicLight,
);

const styleExtension = computed(() => {
  const activeLineHighlight = appTheme.value === "dark" ? "#7c6f64" : "#e0dee9";
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
Vim.defineEx("write", "w", onLocalSave);

// Initialization /////////////////////////////////////////////////////////////////////////////////
const mountEditor = async () => {
  if (!editorMount.value) return;

  const extensions = [basicSetup];

  if (props.typescript) {
    if (!props.noLint) {
      const { lintSource, autocomplete } = await createTypescriptSource(
        props.typescript,
      );

      extensions.push(autocompleteCompartment.of(autocomplete));
      extensions.push(lintCompartment.of(linter(lintSource)));
      extensions.push(lintGutter());
    }

    extensions.push(language.of(CodemirrorJsLang()));
  }

  if (props.json) {
    extensions.push(language.of(CodemirrorJsonLang()));
  }

  const editorState = EditorState.create({
    doc: draftValue.value,
    extensions: extensions.concat([
      themeCompartment.of(codeMirrorTheme.value),
      styleExtensionCompartment.of(styleExtension.value),
      keymap.of([
        indentWithTab,
        { key: "ctrl-s", run: onLocalSave },
        { key: "cmd-s", run: onLocalSave },
      ]),

      readOnly.of(EditorState.readOnly.of(props.disabled)),
      vimCompartment.of(vimEnabled.value ? vim({ status: true }) : []),
      EditorView.updateListener.of(onEditorValueUpdated),
      EditorView.lineWrapping,
    ]),
  });

  view = new EditorView({
    state: editorState,
    parent: editorMount.value,
  });
};

watch(editorMount, mountEditor, { immediate: true });

function onLocalSave() {
  emitUpdatedValue();
  emit("explicitSave");
  return true; // codemirror needs this when used as a "command"
}
</script>

<style>
.cm-editor .cm-content {
  font-size: 14px;
}

.cm-editor .cm-gutter {
  font-size: 14px;
}
</style>
