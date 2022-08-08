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
          v-if="editMode"
          tooltip-text="Re-generate code"
          ignore-text-color
          @click="emit('generate')"
        >
          <slot name="refreshIcon"></slot>
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
import { ref, onMounted, computed, watch } from "vue";
import { EditorState, EditorView, basicSetup } from "@codemirror/basic-setup";
import { yaml } from "@codemirror/legacy-modes/mode/yaml";
import { diff } from "@codemirror/legacy-modes/mode/diff";
import { json } from "@codemirror/legacy-modes/mode/javascript";
import { StreamLanguage, StreamParser } from "@codemirror/stream-parser";
import { keymap } from "@codemirror/view";
import { indentWithTab } from "@codemirror/commands";
import { gruvboxDark } from "cm6-theme-gruvbox-dark";
import { gruvboxLight } from "cm6-theme-gruvbox-light";
import { refFrom } from "vuse-rx/src";
import { Compartment, Extension, StateEffect } from "@codemirror/state";
import { ChangeSetService } from "@/service/change_set";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import { ClipboardCopyIcon } from "@heroicons/vue/solid";
import { ThemeService } from "@/service/theme";
import { Theme } from "@/observable/theme";
import { CodeLanguage } from "@/api/sdf/dal/code_view";

const props = defineProps<{
  componentId: number;
  code: string;

  codeLanguage?: CodeLanguage;
  schemaName?: string;

  // Format: "0.0px"
  fontSize?: string;
  forceTheme?: "dark" | "light";
}>();

const emit = defineEmits(["generate"]);

const editorMount = ref(null);
const view = ref<null | EditorView>(null);
const readOnly = new Compartment();

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());

// This doesn't work on IE, do we care? (is it polyfilled by our build system?)
// RE ^^: https://www.youtube.com/watch?v=Ram7AKbtkGE
const copyCode = () => {
  if (!view.value) return;
  const code = view.value.state.doc.toString().trim();
  navigator.clipboard.writeText(code);
};

// FIXME(nick): for now, we default to "yaml".
const mode = computed((): StreamParser<unknown> => {
  if (props.codeLanguage) {
    if (props.codeLanguage === "diff") {
      return diff;
    } else if (props.codeLanguage === "json") {
      return json;
    }
  }
  return yaml;
});

const forcedTheme = computed((): Extension | null => {
  if (props.forceTheme) {
    if (props.forceTheme === "dark") {
      return gruvboxDark;
    } else if (props.forceTheme === "light") {
      return gruvboxLight;
    }
  }
  return null;
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
        doc: props.code,
        extensions: [
          basicSetup,
          forcedTheme.value ?? currentTheme.value?.value === "dark"
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

// FIXME(nick,victor,wendy): we should try to not reconfigure entire effects when switching themes.
ThemeService.currentTheme().subscribe((theme) => {
  if (view.value) {
    view.value.dispatch({
      effects: StateEffect.reconfigure.of([
        basicSetup,
        forcedTheme.value ?? theme.value === "dark"
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

// Dispatch new code if the prop has changed.
watch(
  computed(() => props.code),
  () => {
    if (!view.value) return;
    view.value.dispatch({
      changes: {
        from: 0,
        to: view.value.state.doc.length,
        insert: props.code,
      },
      effects: readOnly.reconfigure(EditorState.readOnly.of(true)),
    });
  },
);
</script>
