<template>
  <div
    class="w-full h-full border-neutral-300 dark:border-neutral-600 border-x p-2"
  >
    <div ref="editorMount" @keyup.stop @keydown.stop />
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
import { linter, lintGutter } from "@codemirror/lint";
import { createLintSource } from "@/utils/typescriptLinter";
import { useTheme } from "@/composables/injectTheme";
import { changeFunc, funcState, nullEditingFunc } from "./func_state";

const isDevMode = import.meta.env.DEV;

const props = defineProps<{
  funcId: number;
}>();

const funcId = toRef(props, "funcId", -1);
const editingFunc = computed(
  () => funcState.funcs.find((f) => f.id === funcId.value) ?? nullEditingFunc,
);

const editorMount = ref();
const view = ref<EditorView | undefined>();

const language = new Compartment();
const readOnly = new Compartment();
const themeCompartment = new Compartment();
const lintCompartment = new Compartment();
const lintSource = createLintSource();

const appTheme = useTheme();
const codeMirrorTheme = computed(() =>
  appTheme.value === "dark" ? gruvboxDark : basicLight,
);
watch(codeMirrorTheme, () => {
  view.value?.dispatch({
    effects: [themeCompartment.reconfigure(codeMirrorTheme.value)],
  });
});

const mountEditor = async () => {
  const updateListener = EditorView.updateListener.of((update) => {
    if (!update.docChanged) {
      return;
    }
    const newCode = update.view.state.doc.toString();
    changeFunc({ ...editingFunc.value, code: newCode });
  });

  const editorState = EditorState.create({
    doc: editingFunc.value.code,
    extensions: [
      lintCompartment.of(linter(await lintSource)),
      lintGutter(),
      basicSetup,
      language.of(javascript()),
      keymap.of([indentWithTab]),
      themeCompartment.of(codeMirrorTheme.value),
      keymap.of([indentWithTab]),
      readOnly.of(
        EditorState.readOnly.of(!isDevMode && editingFunc.value.isBuiltin),
      ),
      updateListener,
      EditorView.lineWrapping,
    ],
  });

  view.value = new EditorView({
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
