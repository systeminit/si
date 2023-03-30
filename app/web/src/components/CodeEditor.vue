<template>
  <div
    class="w-full h-full border-neutral-300 dark:border-neutral-600 border-x p-2"
  >
    <div
      ref="editorMount"
      class="border border-neutral-300 dark:border-neutral-600"
      @keyup.stop
      @keydown.stop
    />
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
import { useTheme } from "@si/vue-lib/design-system";
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

const { theme: appTheme } = useTheme();
const codeMirrorTheme = computed(() =>
  appTheme.value === "dark" ? gruvboxDark : basicLight,
);
watch(codeMirrorTheme, () => {
  view.dispatch({
    effects: [themeCompartment.reconfigure(codeMirrorTheme.value)],
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
      keymap.of([indentWithTab]),
      readOnly.of(EditorState.readOnly.of(disabled.value)),
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

.cm-activeLine {
  background-color: transparent !important;
}
</style>
