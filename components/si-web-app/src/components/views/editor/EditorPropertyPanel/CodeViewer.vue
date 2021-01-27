<template>
  <div id="code-viewer" class="w-full h-full">
    <div class="w-full h-full">
      <textarea id="codemirror-mount" class="w-full h-full"> </textarea>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState, mapGetters } from "vuex";
import codemirror from "codemirror";
import "codemirror/lib/codemirror.css";
import "codemirror/theme/base16-dark.css";
import "codemirror/mode/yaml/yaml";

import { Node } from "@/api/sdf/model/node";
import _ from "lodash";

interface IData {
  codemirror: null | CodeMirror.Editor;
}

export default Vue.extend({
  name: "CodeViewer",
  data(): IData {
    return {
      codemirror: null,
    };
  },
  computed: {
    ...mapGetters({
      codeProperty: "editor/codeProperty",
    }),
    ...mapState({
      mode: (state: any): any => state.editor.mode,
      selectedNode: (state: any): any => state.editor.node,
    }),
    fieldValue: {
      set(value: any) {
        this.$store.dispatch("editor/entitySet", {
          path: this.codeProperty.path,
          value,
        });
      },
      get(): string {
        if (this.codeProperty) {
          let fieldValue: any = _.cloneDeep(
            _.get(
              this.$store.state.editor.editObject.properties["__baseline"],
              this.codeProperty.path,
            ),
          );
          return fieldValue;
        } else {
          return "# No Code!";
        }
      },
    },
  },
  methods: {
    updateModel(): void {
      if (this.codemirror) {
        if (this.selectedNode) {
          this.codemirror.setValue(this.fieldValue);
          if (this.fieldValue == "# No Code!" || this.mode == "view") {
            this.codemirror.setOption("readOnly", true);
          } else {
            this.codemirror.setOption("readOnly", false);
          }
        }
        this.codemirror.refresh();
      }
    },
  },
  async mounted() {
    await this.$store.dispatch("editor/loadEditObject");
    const mountPoint = document.getElementById(
      "codemirror-mount",
    ) as HTMLTextAreaElement;
    if (mountPoint) {
      const doc = codemirror.fromTextArea(mountPoint, {
        lineNumbers: true,
        viewportMargin: Infinity,
        mode: "yaml",
        theme: "base16-dark",
        screenReaderLabel: "code-mirror-editor",
      });
      doc.setSize("100%", "90%");
      const codeViewerComponent = this;
      doc.on("blur", function () {
        let currentValue = doc.getValue();
        if (currentValue != "# No Code!") {
          codeViewerComponent.fieldValue = currentValue;
        }
      });

      this.codemirror = doc;

      this.updateModel();
    }
  },
  watch: {
    async selectedNode(currentValue: Node) {
      await this.$store.dispatch("editor/loadEditObject");
      this.updateModel();
    },
    async mode(currentMode: string) {
      this.updateModel();
    },
  },
});
</script>
