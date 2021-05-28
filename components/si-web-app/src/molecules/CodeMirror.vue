<template>
  <div class="w-full h-full CodeMirror" ref="cm" />
</template>

<style scoped>
.CodeMirror {
  font-size: 12px;
}
</style>

<script lang="ts">
import Vue from "vue";
import codemirror from "codemirror";
import "codemirror/lib/codemirror.css";
import "codemirror/theme/base16-dark.css";
import "codemirror/mode/yaml/yaml";
import "codemirror/mode/javascript/javascript";
import { PanelEventBus } from "@/atoms/PanelEventBus";

interface Data {
  codeMirror: null | CodeMirror.Editor;
}

export default Vue.extend({
  name: "CodeMirror",
  props: {
    value: { type: String, required: true },
    lineWrapping: { type: Boolean, default: false },
    readOnly: { type: Boolean, default: false },
    noHighlight: { type: Boolean, default: false },
    mode: {
      type: String,
      default: "yaml",
    },
  },
  data(): Data {
    return {
      codeMirror: null,
    };
  },
  created() {
    window.addEventListener("resize", this.setSize);
    PanelEventBus.$on("maximize-full", this.setSize);
    PanelEventBus.$on("maximize-container", this.setSize);
    PanelEventBus.$on("panel-layout-update", this.setSize);
  },
  beforeDestroy() {
    window.removeEventListener("resize", this.setSize);
    PanelEventBus.$off("maximize-full", this.setSize);
    PanelEventBus.$off("maximize-container", this.setSize);
    PanelEventBus.$off("panel-layout-update", this.setSize);
  },
  beforeUpdate() {
    this.setSize();
  },
  mounted() {
    let mode: String | Object;
    let tabSize = 2;
    if (this.mode == "json") {
      mode = { name: "javascript", json: true };
      tabSize = 1;
    } else {
      mode = this.mode;
    }

    let codeMirrorOptions: CodeMirror.EditorConfiguration = {
      tabSize: tabSize,
      value: this.value,
      lineNumbers: true,
      viewportMargin: Infinity,
      theme: "base16-dark",
      lineWrapping: this.lineWrapping,
      readOnly: this.readOnly,
      mode: mode, //works for YAML but should be an option!
    };
    if (this.noHighlight) {
      codeMirrorOptions["mode"] = null;
    }
    const codeMirror = codemirror(
      this.$refs.cm as HTMLElement,
      codeMirrorOptions,
    );
    this.codeMirror = codeMirror;
    this.setSize();
  },
  watch: {
    value(newValue: string) {
      if (this.codeMirror) {
        this.codeMirror.setValue(newValue);
        this.setSize();
      }
    },
  },
  methods: {
    setSize() {
      if (this.codeMirror) {
        const element = this.$refs.cm as HTMLElement;
        const boundingRect = element.getBoundingClientRect();
        this.codeMirror.setSize(boundingRect.width, boundingRect.height);
      }
    },
    onMaximize() {
      console.log("max");
      this.setSize();
    },
  },
});
</script>
