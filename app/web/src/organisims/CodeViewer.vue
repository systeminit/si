<template>
  <div v-if="props.componentId" class="flex flex-col w-full overflow-hidden">
    <div
      class="flex flex-row items-center justify-between h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div class="text-lg">Component ID {{ props.componentId }} Code</div>
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
import { ref, onMounted, toRefs } from "vue";
import { EditorState, EditorView, basicSetup } from "@codemirror/basic-setup";
import { yaml } from "@codemirror/legacy-modes/mode/yaml";
import { StreamLanguage } from "@codemirror/stream-parser";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { gruvboxDark } from "cm6-theme-gruvbox-dark";
import { fromRef, refFrom } from "vuse-rx/src";
import { combineLatestWith, switchMap, tap } from "rxjs";
import { ComponentService } from "@/service/component";
import _ from "lodash";
import { GlobalErrorService } from "@/service/global_error";
import { Compartment } from "@codemirror/state";

const props = defineProps<{
  componentId: number;
}>();
const { componentId } = toRefs(props);
const componentId$ = fromRef(componentId, { immediate: true });
const editorMount = ref(null);
const view = ref<null | EditorView>(null);
const view$ = fromRef(view, { immediate: true });
const readOnly = new Compartment();

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
          readOnly.of(EditorState.readOnly.of(false)),
        ],
      }),
      parent: editorMount.value,
    });
  }
});

// @ts-ignore
const _code = refFrom(
  componentId$.pipe(
    switchMap((componentId) => {
      return ComponentService.getCode({ componentId });
    }),
    combineLatestWith(view$),
    tap(([reply, view]) => {
      if (reply.error) {
        GlobalErrorService.set(reply);
      } else {
        if (view) {
          // Eventually, we should support multiple code outputs
          if (reply.codeViews.length > 0) {
            let insert = reply.codeViews[0].code;
            view.dispatch({
              changes: {
                from: 0,
                to: view.state.doc.length,
                insert,
              },
              effects: readOnly.reconfigure(EditorState.readOnly.of(false)),
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
