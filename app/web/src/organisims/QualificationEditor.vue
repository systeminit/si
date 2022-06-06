<template>
  <div class="flex flex-col w-full h-full overflow-hidden">
    <div
      class="flex justify-between h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-md">Qualification {{ code?.prototype?.title }} Code</div>

      <div class="flex">
        <SiButtonIcon v-if="editMode" tooltip-text="Save code">
          <SaveIcon />
        </SiButtonIcon>

        <SiButtonIcon
          class="ml-2"
          tooltip-text="Close Editor"
          @click="save().then(() => emit('close'))"
        >
          <XCircleIcon />
        </SiButtonIcon>
      </div>
    </div>
    <div class="w-full h-full overflow-auto">
      <div
        ref="editorMount"
        class="w-full h-full"
        @keyup.stop
        @keydown.stop
      ></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import _ from "lodash";
import * as Rx from "rxjs";
import { ref, onUnmounted, onMounted, toRefs, Ref } from "vue";
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
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import { SaveIcon, XCircleIcon } from "@heroicons/vue/solid";

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
  if (code.value && code.value.code !== newCode) {
    QualificationService.setCode({
      prototypeId: prototypeId.value,
      code: newCode,
      systemId: system?.id,
    }).subscribe((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
      }
    });
  }
}

const code: Ref<GetCodeResponse | undefined> = refFrom<GetCodeResponse>(
  Rx.combineLatest([prototypeId$, system$]).pipe(
    Rx.switchMap(([prototypeId]) => {
      return QualificationService.getCode({ prototypeId });
    }),
    Rx.combineLatestWith(view$),
    Rx.switchMap(([reply, view]) => {
      if (reply.error) {
        GlobalErrorService.set(reply);
        return Rx.from([]);
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
        return Rx.from([reply]);
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
