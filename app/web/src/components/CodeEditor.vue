<template>
  <div class="w-full h-full">
    <div class="absolute right-xs top-xs">
      <VButton2
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
import { computed, onMounted, ref, toRef, watch } from "vue";
import { basicSetup, EditorView } from "codemirror";
import { Compartment, EditorState } from "@codemirror/state";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { gruvboxDark } from "cm6-theme-gruvbox-dark";
import { basicLight } from "cm6-theme-basic-light";
import { javascript } from "@codemirror/lang-javascript";
import { json } from "@codemirror/lang-json";
import { linter, lintGutter } from "@codemirror/lint";
import { useTheme, VButton2 } from "@si/vue-lib/design-system";
import { vim } from "@replit/codemirror-vim";
import storage from "local-storage-fallback";
import { createLintSource } from "@/utils/typescriptLinter";

const props = defineProps<{
  modelValue: string;
  disabled?: boolean;
  json?: boolean;
  typescript?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", v: string): void;
  (e: "change", v: string): void;
}>();

const editorMount = ref();
let view: EditorView;

const modelValue = toRef(props, "modelValue", "");
const disabled = toRef(props, "disabled", false);
const useJson = toRef(props, "json", false);
const useTypescript = toRef(props, "typescript", false);
const currentCode = ref<string>(modelValue.value ?? "");

watch(
  () => modelValue.value,
  async (currentMv) => {
    currentCode.value = currentMv ?? "";
    const currentDoc = view?.state.doc.toString();

    // We only care about this if the code changes from outside the editor itself
    // and we didn't just switch to a new doc. This condition prevents a cycle
    // with the updateListener
    if (currentDoc === currentCode.value || typeof view === "undefined") {
      return;
    }

    const updateTransaction = view.state.update({
      changes: {
        from: 0,
        to: view.state.doc.length,
        insert: currentCode.value,
      },
    });
    view.update([updateTransaction]);
  },
);

const language = new Compartment();
const readOnly = new Compartment();
const themeCompartment = new Compartment();
const lintCompartment = new Compartment();
const styleExtensionCompartment = new Compartment();
const vimCompartment = new Compartment();

const { theme: appTheme } = useTheme();
const codeMirrorTheme = computed(() =>
  appTheme.value === "dark" ? gruvboxDark : basicLight,
);
const styleExtension = computed(() => {
  const activeLineHighlight = appTheme.value === "dark" ? "#7c6f64" : "#e0dee9";
  return EditorView.theme({
    "&": { height: "100%" },
    ".cm-scroller": { overflow: "auto" },
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

// Enable/disable vim mode dynamically
// TODO(nick,zack): put this into a library (maybe?)
const VIM_MODE_STORAGE_KEY = "SI:VIM_MODE";
const vimEnabledDefault = (): boolean => {
  return storage.getItem(VIM_MODE_STORAGE_KEY) === "true";
};
const vimEnabled = ref(vimEnabledDefault());
watch(vimEnabled, (useVim) => {
  storage.setItem(VIM_MODE_STORAGE_KEY, useVim ? "true" : "false");
  view.dispatch({
    effects: [vimCompartment.reconfigure(useVim ? vim({ status: true }) : [])],
  });
});

const mountEditor = async () => {
  currentCode.value = modelValue.value ?? "";
  const updateListener = EditorView.updateListener.of((update) => {
    if (!update.docChanged) return;
    const updatedCode = update.view.state.doc.toString();
    emit("update:modelValue", updatedCode);
    emit("change", updatedCode);
  });

  const extensions = [basicSetup];

  if (useTypescript.value) {
    const lintSource = createLintSource();
    extensions.push(lintCompartment.of(linter(await lintSource)));
    extensions.push(lintGutter());
    extensions.push(language.of(javascript()));
  }

  if (useJson.value) {
    extensions.push(language.of(json()));
  }

  const editorState = EditorState.create({
    doc: currentCode.value,
    extensions: extensions.concat([
      keymap.of([indentWithTab]),
      themeCompartment.of(codeMirrorTheme.value),
      styleExtensionCompartment.of(styleExtension.value),
      keymap.of([indentWithTab]),
      readOnly.of(EditorState.readOnly.of(disabled.value)),
      vimCompartment.of(vimEnabled.value ? vim({ status: true }) : []),
      updateListener,
      EditorView.lineWrapping,
    ]),
  });

  view = new EditorView({
    state: editorState,
    parent: editorMount.value,
  });
};

onMounted(() => {
  if (editorMount.value) {
    mountEditor();
  }
});
</script>

<style>
.cm-editor .cm-content {
  font-size: 14px;
}

.cm-editor .cm-gutter {
  font-size: 14px;
}
</style>
