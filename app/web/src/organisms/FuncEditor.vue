<template>
  <!-- old save and discard buttons, move them! -->
  <!--  <div class="mb-3 flex items-center gap-x-[0.9375rem]">-->
  <!--    <TertiaryDestructiveButtonXSmall-->
  <!--      label="Discard"-->
  <!--      icon-style="left"-->
  <!--      icon="x"-->
  <!--      @click="discardChanges"-->
  <!--    />-->
  <!--    <PrimarySuccessButtonXSmall label="Save" icon-style="left" />-->
  <!--  </div>-->
  <div class="w-full border-neutral-300 dark:border-neutral-600 p-2">
    <div ref="editorMount" class="w-full h-full" @keyup.stop @keydown.stop />
  </div>
</template>

<script lang="ts" setup>
import { onMounted, ref, toRef } from "vue";
import { EditorState, StateField } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap } from "@codemirror/commands";
import {
  EditingFunc,
  funcEdit$,
  funcStream$,
  nullEditingFunc,
} from "@/observable/func_editor";
import { map } from "rxjs/operators";

import { refFrom } from "vuse-rx/src";

import PrimarySuccessButtonXSmall from "@/molecules/PrimarySuccessButtonXSmall.vue";
import TertiaryDestructiveButtonXSmall from "@/molecules/TertiaryDestructiveButtonXSmall.vue";

const props = defineProps<{
  funcId: number;
}>();

const funcId = toRef(props, "funcId", -1);
const editingFunc = refFrom<EditingFunc>(
  funcStream$.pipe(
    map(
      (editingFunc) =>
        editingFunc?.find((f) => f.id == funcId.value) ?? nullEditingFunc,
    ),
  ),
  nullEditingFunc,
);

const editorMount = ref();
const view = ref<EditorView | undefined>();

const onCodeUpdate = StateField.define({
  create: () => 0,
  update: (value, tr) => {
    if (!tr.docChanged) {
      return value;
    }

    if (view.value) {
      const code = view.value.state.doc.toString();
      funcEdit$.next({
        type: "change",
        func: {
          ...editingFunc.value.modifiedFunc,
          code,
        },
      });
    }

    return value + 1;
  },
});

const discardChanges = () => {
  view.value?.dispatch({
    changes: {
      from: 0,
      to: view.value.state.doc.length,
      insert: editingFunc.value.origFunc.code,
    },
  });
};

const mountEditor = () => {
  const editorState = EditorState.create({
    doc: editingFunc.value.modifiedFunc.code,
    extensions: [keymap.of(defaultKeymap), onCodeUpdate],
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
