<template>
  <div class="flex flex-col w-full h-full px-4 py-2 overflow-hidden mt-2">
    <div @keyup.stop @keydown.stop>
      <SiTextBox2
        id="qualification-name"
        v-model="name"
        class="mb-2 p-2 w-full"
        title="qualification name"
        required
        @blur="save()"
      />
    </div>

    <div class="overflow-auto">
      <div
        ref="editorMount"
        class="w-full h-full"
        @keyup.stop
        @keydown.stop
      ></div>
    </div>

    <div class="flex mt-2 flex-row-reverse">
      <SiButton
        class="ml-2"
        label="close"
        size="xs"
        icon="cancel"
        kind="cancel"
        @click="save().then(() => emit('close'))"
      />

      <SiButton
        v-if="editMode"
        label="save"
        size="xs"
        icon="save"
        kind="save"
        @click="save().then(() => emit('close'))"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import _ from "lodash";
import * as Rx from "rxjs";
import { ref, onUnmounted, onMounted, toRefs } from "vue";
import { EditorState, EditorView, basicSetup } from "@codemirror/basic-setup";
import { javascript } from "@codemirror/legacy-modes/mode/javascript";
import { StreamLanguage } from "@codemirror/stream-parser";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { gruvboxDark } from "cm6-theme-gruvbox-dark";
import { fromRef, refFrom } from "vuse-rx/src";
import { QualificationService } from "@/service/qualification";
import { GetCodeResponse } from "@/service/qualification/get_code";
import { GlobalErrorService } from "@/service/global_error";
import { Compartment } from "@codemirror/state";
import { ChangeSetService } from "@/service/change_set";
import { system$ } from "@/observable/system";
import SiButton from "@/atoms/SiButton.vue";
import SiTextBox2 from "@/atoms/SiTextBox2.vue";

const emit = defineEmits(["close"]);

const props = defineProps<{
  prototypeId: number;
}>();
const { prototypeId } = toRefs(props);
const prototypeId$ = fromRef(prototypeId, {
  immediate: true,
});
const editorMount = ref(null);
const view = ref<null | EditorView>(null);
const view$ = fromRef(view, { immediate: true });
const readOnly = new Compartment();

const name = ref("");

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());

onMounted(() => {
  if (editorMount.value) {
    const fixedHeightEditor = EditorView.theme({
      "&": { height: "100%" },
      ".cm-scroller": { overflow: "auto" },
      ".cm-content": { "white-space": "pre-wrap" },
    });

    view.value = new EditorView({
      state: EditorState.create({
        extensions: [
          basicSetup,
          gruvboxDark,
          fixedHeightEditor,
          keymap.of([indentWithTab]),
          StreamLanguage.define(javascript),
          readOnly.of(EditorState.readOnly.of(true)),
        ],
      }),
      parent: editorMount.value,
    });
    view.value.contentDOM.addEventListener("blur", save);
  }
});

onUnmounted(() => {
  if (!view.value) return;
  view.value.contentDOM.removeEventListener("blur", save);
});

async function save() {
  if (!view.value) return;

  const system = await Rx.firstValueFrom(system$);

  const newCode = view.value.state.doc.toString().trim();
  if (
    code.value &&
    name.value &&
    (code.value.code !== newCode || code.value.prototype.title !== name.value)
  ) {
    QualificationService.setCode({
      prototypeId: prototypeId.value,
      prototypeTitle: name.value,
      code: newCode,
      systemId: system?.id,
    }).subscribe((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
      }
    });
  }
}

const code = refFrom<GetCodeResponse | undefined>(
  Rx.combineLatest([prototypeId$, system$]).pipe(
    Rx.switchMap(([prototypeId]) => {
      return QualificationService.getCode({ prototypeId });
    }),
    Rx.combineLatestWith(view$),
    Rx.map(([reply, view]) => {
      if (reply.error) {
        GlobalErrorService.set(reply);
        return;
      } else if (reply) {
        if (view) {
          view.dispatch({
            changes: {
              from: 0,
              to: view.state.doc.length,
              insert: reply.code,
            },
            effects: readOnly.reconfigure(
              EditorState.readOnly.of(!editMode.value),
            ),
          });
        }
        name.value = reply.prototype.title;
        return reply as GetCodeResponse;
      }
    }),
  ),
);
</script>

<style scoped>
.property-section-bg-color {
  background-color: #292c2d;
}
</style>
