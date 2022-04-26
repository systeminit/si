<template>
  <div v-if="props.componentId" class="flex flex-col w-full overflow-hidden">
    <div
      class="flex flex-row items-center justify-between h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">Component ID {{ props.componentId }} Code</div>

      <button class="ml-2 text-base" @click="copyCode">
        <VueFeather type="copy" size="1em" />
      </button>
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
import { ref, onMounted, toRefs } from "vue";
import VueFeather from "vue-feather";
import { EditorState, EditorView, basicSetup } from "@codemirror/basic-setup";
import { yaml } from "@codemirror/legacy-modes/mode/yaml";
import { StreamLanguage } from "@codemirror/stream-parser";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { gruvboxDark } from "cm6-theme-gruvbox-dark";
import { fromRef, refFrom, untilUnmounted } from "vuse-rx/src";
import { ComponentService } from "@/service/component";
import { eventCodeGenerated$ } from "@/observable/code";
import { GlobalErrorService } from "@/service/global_error";
import { Compartment } from "@codemirror/state";
import { system$ } from "@/observable/system";

const props = defineProps<{
  componentId: number;
}>();
const { componentId } = toRefs(props);
const componentId$ = fromRef(componentId, { immediate: true });
const editorMount = ref(null);
const view = ref<null | EditorView>(null);
const view$ = fromRef(view, { immediate: true });
const readOnly = new Compartment();

// This doesn't work on IE, do we care? (is it polyfilled by our build system?)
const copyCode = () => {
  if (view.value?.state.doc) {
    navigator.clipboard.writeText(view.value.state.doc.text.join("\n"));
  }
};

onMounted(() => {
  if (editorMount.value) {
    const fixedHeightEditor = EditorView.theme({
      "&": { height: "100%" },
      ".cm-scroller": { overflow: "auto" },
    });

    view.value = new EditorView({
      state: EditorState.create({
        extensions: [
          basicSetup,
          gruvboxDark,
          fixedHeightEditor,
          keymap.of([indentWithTab]),
          StreamLanguage.define(yaml),
          readOnly.of(EditorState.readOnly.of(true)),
        ],
      }),
      parent: editorMount.value,
    });
  }
});

const codeGenerated$ = new Rx.ReplaySubject<true>();
codeGenerated$.next(true); // We must fetch on setup
eventCodeGenerated$.pipe(untilUnmounted).subscribe(async (codeGenerationId) => {
  const system = await Rx.firstValueFrom(system$);
  const data = codeGenerationId?.payload.data;
  const sameComponent = props.componentId === data?.componentId;
  const sameSystem = (system?.id ?? -1) === data?.systemId;
  if (sameComponent && sameSystem) {
    codeGenerated$.next(true);
  }
});

// @ts-ignore
const _code = refFrom(
  Rx.combineLatest([componentId$, system$, codeGenerated$]).pipe(
    Rx.switchMap(([componentId]) => {
      if (componentId) {
        return ComponentService.getCode({ componentId });
      } else {
        return Rx.from([null]);
      }
    }),
    Rx.combineLatestWith(view$),
    Rx.tap(([reply, view]) => {
      if (reply?.error) {
        GlobalErrorService.set(reply);
      } else if (reply) {
        if (view) {
          // Eventually, we should support multiple code outputs
          if (reply.codeViews.length > 0) {
            let insert =
              reply.codeViews[0].code ?? "# Generating code, wait a bit...";
            view.dispatch({
              changes: {
                from: 0,
                to: view.state.doc.length,
                insert,
              },
              effects: readOnly.reconfigure(EditorState.readOnly.of(true)),
            });
          } else {
            view.dispatch({
              changes: {
                from: 0,
                to: view.state.doc.length,
                insert: "# No code is better than no code! :)",
              },
              effects: readOnly.reconfigure(EditorState.readOnly.of(true)),
            });
          }
        }
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
