<template>
  <div class="flex w-full max-full overflow-hidden" @keydown.stop @keyup.stop>
    <div :id="editorId" class="w-full h-full"></div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import * as monaco from "monaco-editor";
import _ from "lodash";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";

interface Data {
  editorId: string;
  codeValue: string;
}

declare module "vue/types/options" {
  interface ComponentOptions<V extends Vue> {
    monaco?: monaco.editor.ICodeEditor;
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
  },
});
</script>
