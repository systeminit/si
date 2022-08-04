<template>
  <div v-if="props.componentId" class="flex flex-col w-full overflow-hidden">
    <div
      class="flex flex-row items-center justify-between h-10 px-6 py-2 text-base align-middle"
    >
      <div v-if="props.schemaName" class="text-lg">
        {{ props.schemaName }}
      </div>
      <div v-else class="text-lg">
        Component ID {{ props.componentId }} Code
      </div>

      <div class="flex">
        <SiButtonIcon
          tooltip-text="Copy code to clipboard"
          ignore-text-color
          @click="copyCode"
        >
          <ClipboardCopyIcon />
        </SiButtonIcon>

        <SiButtonIcon
          v-if="editMode && !props.diffMode"
          tooltip-text="Re-generate code"
          ignore-text-color
          @click="generateCode"
        >
          <RefreshIcon :class="refreshClasses" />
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
import { ref, onMounted, toRefs, computed } from "vue";
import { EditorState, EditorView, basicSetup } from "@codemirror/basic-setup";
import { yaml } from "@codemirror/legacy-modes/mode/yaml";
import { diff } from "@codemirror/legacy-modes/mode/diff";
import { StreamLanguage, StreamParser } from "@codemirror/stream-parser";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { gruvboxDark } from "cm6-theme-gruvbox-dark";
import { gruvboxLight } from "cm6-theme-gruvbox-light";
import { fromRef, refFrom, untilUnmounted } from "vuse-rx/src";
import { ComponentService } from "@/service/component";
import { eventCodeGenerated$ } from "@/observable/code";
import { GlobalErrorService } from "@/service/global_error";
import { Compartment, Extension, StateEffect } from "@codemirror/state";
import { ChangeSetService } from "@/service/change_set";
import { system$ } from "@/observable/system";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import { ClipboardCopyIcon, RefreshIcon } from "@heroicons/vue/solid";
import { ThemeService } from "@/service/theme";
import { Theme } from "@/observable/theme";

const props = defineProps<{
  componentId: number;
  schemaName?: string;
  // FIXME(nick): remove the need for diff mode and make the component more configurable.
  diffMode?: boolean;
  // Format: "0.0px"
  fontSize?: string;
}>();
const { componentId } = toRefs(props);
const componentId$ = fromRef(componentId, { immediate: true });
const editorMount = ref(null);
const view = ref<null | EditorView>(null);
const view$ = fromRef(view, { immediate: true });
const readOnly = new Compartment();

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());

// This doesn't work on IE, do we care? (is it polyfilled by our build system?)
// RE ^^: https://www.youtube.com/watch?v=Ram7AKbtkGE
const copyCode = () => {
  if (!view.value) return;
  const code = view.value.state.doc.toString().trim();
  navigator.clipboard.writeText(code);
};

// FIXME(nick): base mode off of the "CodeLanguage" type coming back.
const mode = computed((): StreamParser<unknown> => {
  if (props.diffMode) {
    return diff;
  }
  return yaml;
});

// FIXME(nick): make this more configurable.
const fixedHeightEditor = computed((): Extension => {
  if (props.fontSize) {
    return EditorView.theme({
      "&": { height: "100%", fontSize: props.fontSize },
      ".cm-scroller": { overflow: "auto" },
    });
  }
  return EditorView.theme({
    "&": { height: "100%" },
    ".cm-scroller": { overflow: "auto" },
  });
});

const currentTheme = refFrom<Theme>(ThemeService.currentTheme());

onMounted(() => {
  if (editorMount.value) {
    view.value = new EditorView({
      state: EditorState.create({
        extensions: [
          basicSetup,
          props.diffMode
            ? gruvboxDark
            : currentTheme.value?.value === "dark"
            ? gruvboxDark
            : gruvboxLight,
          fixedHeightEditor.value,
          keymap.of([indentWithTab]),
          StreamLanguage.define(mode.value),
          readOnly.of(EditorState.readOnly.of(true)),
        ],
      }),
      parent: editorMount.value,
    });
  }
});

// FIXME(nick,victor,wendy): do not reconfigure entire state when switching themes.
ThemeService.currentTheme().subscribe((theme) => {
  if (view.value) {
    view.value.dispatch({
      effects: StateEffect.reconfigure.of([
        basicSetup,
        props.diffMode
          ? gruvboxDark
          : theme.value === "dark"
          ? gruvboxDark
          : gruvboxLight,
        fixedHeightEditor.value,
        keymap.of([indentWithTab]),
        StreamLanguage.define(mode.value),
        readOnly.of(EditorState.readOnly.of(true)),
      ]),
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
        if (props.diffMode) {
          return ComponentService.getDiff({ componentId });
        }
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

const currentSyncAnimate = ref<boolean>(false);
const refreshClasses = computed(() => {
  const classes: { [key: string]: boolean } = {};
  if (currentSyncAnimate.value) {
    classes["animate-spin"] = true;
    classes["transform"] = true;
    classes["rotate-180"] = true;
  } else {
    classes["animate-spin"] = false;
    classes["transform"] = false;
    classes["rotate-180"] = false;
  }
  return classes;
});

const generateCode = () => {
  currentSyncAnimate.value = true;
  ComponentService.generateCode({
    componentId: props.componentId,
  }).subscribe((reply) => {
    currentSyncAnimate.value = false;
    if (reply.error) {
      GlobalErrorService.set(reply);
    } else if (!reply.success) {
      GlobalErrorService.set({
        error: {
          statusCode: 42,
          code: 42,
          message: "Code generation failed silently",
        },
      });
    }
  });
};
</script>
