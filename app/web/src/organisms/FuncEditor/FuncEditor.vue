<template>
  <div
    class="w-full h-full border-neutral-300 dark:border-neutral-600 border-x px-2"
  >
    <div ref="editorMount" @keyup.stop @keydown.stop />
  </div>
</template>

<script lang="ts" setup>
import { onMounted, ref, toRef, computed, watch } from "vue";
import { basicSetup, EditorView } from "@codemirror/basic-setup";
import { EditorState, Compartment } from "@codemirror/state";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { funcState, changeFunc, nullEditingFunc } from "./func_state";
import { gruvboxDark } from "cm6-theme-gruvbox-dark";
import { basicLight } from "cm6-theme-basic-light";
import { ThemeService } from "@/service/theme";
import { Theme } from "@/observable/theme";
import { refFrom } from "vuse-rx";
import { javascript } from "@codemirror/lang-javascript";

const props = defineProps<{
  funcId: number;
}>();

const funcId = toRef(props, "funcId", -1);
const editingFunc = computed(
  () => funcState.funcs.find((f) => f.id == funcId.value) ?? nullEditingFunc,
);

const currentTheme = refFrom<Theme>(ThemeService.currentTheme());
const editorMount = ref();
const view = ref<EditorView | undefined>();

const language = new Compartment();
const readOnly = new Compartment();
const themeCompartment = new Compartment();

watch(
  () => currentTheme.value,
  (newTheme) => {
    view.value?.dispatch({
      effects: [
        themeCompartment.reconfigure(
          newTheme?.value === "dark" ? gruvboxDark : basicLight,
        ),
      ],
    });
    console.log(newTheme, view.value?.state);
  },
);

const mountEditor = () => {
  const updateListener = EditorView.updateListener.of((update) => {
    if (!update.docChanged) {
      return;
    }
    const newCode = update.view.state.doc.toString();
    changeFunc({ ...editingFunc.value.modifiedFunc, code: newCode });
  });

  const editorState = EditorState.create({
    doc: editingFunc.value.modifiedFunc.code,
    extensions: [
      basicSetup,
      language.of(javascript()),
      keymap.of([indentWithTab]),
      themeCompartment.of(
        currentTheme.value?.value === "dark" ? gruvboxDark : basicLight,
      ),
      keymap.of([indentWithTab]),
      readOnly.of(
        EditorState.readOnly.of(editingFunc.value.origFunc.isBuiltin),
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
