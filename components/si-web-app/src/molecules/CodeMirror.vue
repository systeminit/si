<template>
  <div class="h-full" ref="cm" />
</template>

<style scoped>
.CodeMirror {
  height: auto;
}
</style>

<script lang="ts">
import Vue from "vue";
import codemirror from "codemirror";
import "codemirror/lib/codemirror.css";
import "codemirror/theme/base16-dark.css";
import "codemirror/mode/yaml/yaml";

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
  },
  data(): Data {
    return {
      codeMirror: null,
    };
  },
  mounted() {
    let codeMirrorOptions: CodeMirror.EditorConfiguration = {
      value: this.value,
      lineNumbers: true,
      viewportMargin: Infinity,
      theme: "base16-dark",
      lineWrapping: this.lineWrapping,
      readOnly: this.readOnly,
    };
    if (this.noHighlight) {
      codeMirrorOptions["mode"] = null;
    }
    const codeMirror = codemirror(
      this.$refs.cm as HTMLElement,
      codeMirrorOptions,
    );
    this.codeMirror = codeMirror;
  },
  watch: {
    value(newValue: string) {
      if (this.codeMirror) {
        this.codeMirror.setValue(newValue);
      }
    },
  },
});
</script>
