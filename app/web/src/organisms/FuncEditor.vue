<template>
  <div class="overflow-y-scroll">
    <SiTextBox2
      id="handler"
      v-model="handler"
      @blur="setHandler"
      title="Entrypoint"
    />
  </div>
  <div>
    <SiTextBox2 id="name" v-model="name" title="Name" @blur="setName" />
  </div>
  <div class="mb-3 flex items-center gap-x-[0.9375rem]">
    <TertiaryDestructiveButtonXSmall
      label="Discard"
      icon-style="left"
      icon="x"
      @click="discardChanges"
    />
    <PrimarySuccessButtonXSmall
      label="Save"
      icon-style="left"
      :disabled="!isDirty"
    />
  </div>
  <div>
    <div ref="editorMount" class="w-full h-full" @keyup.stop @keydown.stop />
  </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref, toRef } from "vue";
import { EditorState, StateField } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap } from "@codemirror/commands";
import {
  EditingFunc,
  editingFuncs$,
  selectedTab$,
} from "@/observable/func_editor";

import { refFrom, fromRef } from "vuse-rx/src";
import { combineLatest, of } from "rxjs";
import { mergeMap } from "rxjs/operators";
import { FuncBackendKind } from "@/api/sdf/dal/func";

import SiTextBox2 from "@/atoms/SiTextBox2.vue";
import PrimarySuccessButtonXSmall from "@/molecules/PrimarySuccessButtonXSmall.vue";
import TertiaryDestructiveButtonXSmall from "@/molecules/TertiaryDestructiveButtonXSmall.vue";

const props = defineProps<{
  funcId: number;
}>();

const nullFunc: EditingFunc = {
  origFunc: {
    id: 0,
    handler: undefined,
    kind: FuncBackendKind.Unset,
    name: "",
    code: "",
  },
  modifiedFunc: {
    id: 0,
    handler: undefined,
    kind: FuncBackendKind.Unset,
    name: "",
    code: "",
  },
  id: 0,
};

const editingFuncs = refFrom<EditingFunc[]>(editingFuncs$, []);

const funcId = toRef(props, "funcId", -1);
const editingFunc = computed(
  () => editingFuncs.value.find((f) => f.id === funcId.value) ?? nullFunc,
);

const handler = ref<string>(editingFunc.value.modifiedFunc.handler ?? "");
const name = ref<string>(editingFunc.value.modifiedFunc.name);

const emit = defineEmits<{
  (e: "updatedName", v: string): void;
  (e: "updatedHandler", v: string): void;
  (e: "updatedCode", v: string): void;
}>();

const setName = () => emit("updatedName", handler.value);
const setHandler = () => emit("updatedHandler", name.value);
const setCode = (code: string) => emit("updatedCode", code);

const editorMount = ref();
const view = ref<EditorView | undefined>();

const isDirty = refFrom<boolean>(
  combineLatest([editingFuncs$]).pipe(
    mergeMap(([editingFuncs]) =>
      of(
        editingFuncs[props.funcId]?.modifiedFunc.handler !==
          editingFuncs[props.funcId]?.origFunc.handler ||
          editingFuncs[props.funcId]?.modifiedFunc.name !==
            editingFuncs[props.funcId]?.origFunc.name ||
          editingFuncs[props.funcId]?.modifiedFunc.code !==
            editingFuncs[props.funcId]?.origFunc.code,
      ),
    ),
  ),
  false,
);

const onCodeUpdate = StateField.define({
  create: () => 0,
  update: (value, tr) => {
    if (!tr.docChanged) {
      return value;
    }

    if (view.value) {
      setCode(view.value.state.doc.toString());
    }

    console.log("foo");

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
  console.log(props);
});
</script>
