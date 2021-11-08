<template>
  <div class="flex w-full overflow-hidden max-full" @keydown.stop @keyup.stop>
    <div :id="editorId" class="w-full h-full"></div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import * as monaco from "monaco-editor";
import { editor as MonacoEditor } from "monaco-editor";
import _ from "lodash";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { CodeDecorationItem } from "si-entity";

interface Data {
  editorId: string;
  codeValue: string;
}

declare module "vue/types/options" {
  interface ComponentOptions<V extends Vue> {
    monaco?: monaco.editor.ICodeEditor;
    decorations?: string[];
  }
}

export default Vue.extend({
  name: "Monaco",
  props: {
    value: {
      type: String,
      required: true,
    },
    readOnly: {
      type: Boolean,
      default: false,
    },
    codeDecorations: {
      type: Array as PropType<CodeDecorationItem[]>,
      default: [],
    },
  },
  data(): Data {
    let codeValue;
    if (this.value) {
      codeValue = this.value;
    } else {
      codeValue = "No code is the best code!";
    }
    return {
      editorId: _.uniqueId("monaco_"),
      codeValue,
    };
  },
  mounted() {
    const editorElem = document.getElementById(this.editorId);
    if (editorElem) {
      let editor = monaco.editor.create(editorElem, {
        language: "yaml",
        automaticLayout: true,
        theme: "vs-dark",
        wordWrap: "on",
        wrappingIndent: "indent",
        roundedSelection: true,
        autoIndent: "advanced",
        cursorBlinking: "smooth",
        glyphMargin: true,
        lightbulb: { enabled: true },
        matchBrackets: "always",
        readOnly: this.readOnly,
      });
      editor.setValue(this.value);
      this.$options.monaco = editor;
      this.$options.decorations = [];
      editor.onDidBlurEditorText(() => {
        this.$emit("blur");
      });
      editor.onDidChangeModelContent(_e => {
        let newValue = editor.getValue();
        this.$emit("input", newValue);
      });
    } else {
      emitEditorErrorMessage(
        `cannot find editor element for monaco: ${this.editorId}`,
      );
    }
  },
  methods: {
    setValue(code: string) {
      this.codeValue = code;
      if (this.$options.monaco) {
        this.$options.monaco.setValue(this.codeValue);
      }
    },
  },
  watch: {
    value: {
      handler(newValue: string) {
        this.setValue(newValue);
      },
    },
    readOnly: {
      handler(newValue: boolean) {
        if (this.$options.monaco) {
          this.$options.monaco.updateOptions({ readOnly: newValue });
        }
      },
    },
    codeDecorations: {
      immediate: true,
      handler(newValue: CodeDecorationItem[]) {
        if (this.$options.monaco && _.isArray(this.$options.decorations)) {
          const newDecorations: Array<MonacoEditor.IModelDeltaDecoration> = [];
          for (const codeItem of newValue) {
            const range = new monaco.Range(
              codeItem.startLine,
              codeItem.startCol,
              codeItem.endLine,
              codeItem.endCol,
            );
            const options: MonacoEditor.IModelDecorationOptions = {};
            if (codeItem.type == "line") {
              options.isWholeLine = true;
              if (codeItem.kind == "driven") {
                if (codeItem.source == "inferred") {
                  options.linesDecorationsClassName = "inferredLineDecoration";
                } else {
                  options.linesDecorationsClassName = "manualLineDecoration";
                }
              } else if (codeItem.kind == "changed") {
                options.linesDecorationsClassName = "changedLineDecoration";
              } else if (codeItem.kind == "qualification") {
                options.linesDecorationsClassName =
                  "qualificationLineDecoration";
              }
            }
            newDecorations.push({
              range,
              options,
            });
          }
          this.$options.decorations = this.$options.monaco.deltaDecorations(
            this.$options.decorations,
            newDecorations,
          );
        }
      },
    },
  },
});
</script>

<style>
.inferredLineDecoration {
  background: #69c6e3;
  width: 2px !important;
  margin-left: 3px;
}
.manualLineDecoration {
  background: #5b6163;
  width: 2px !important;
  margin-left: 3px;
}
.changedLineDecoration {
  background: #ce7f3e;
  width: 2px !important;
  margin-left: 7px;
}
.qualificationLineDecoration {
  background: rgba(248, 113, 113, 1);
  width: 2px !important;
  margin-left: 11px;
}
</style>
