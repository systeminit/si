<template>
  <div id="code-viewer" class="flex w-full h-full">
    <div id="monaco-mount" class="flex-auto w-full h-full" />
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState, mapGetters } from "vuex";
import * as monaco from "monaco-editor";

import { Node } from "@/api/sdf/model/node";
import _ from "lodash";

interface Data {
  editor: monaco.editor.IEditor | undefined;
}

export default Vue.extend({
  name: "CodeViewer",
  data(): Data {
    return {
      editor: undefined,
    };
  },
  computed: {
    ...mapGetters({
      codeProperty: "editor/codeProperty",
    }),
    ...mapState({
      selectedNode: (state: any): any => state.editor.node,
    }),
    fieldValue: {
      set(value: any) {
        console.log("setting the new value", {
          value,
          codePropertyPath: this.codeProperty.path,
        });

        this.$store.dispatch("editor/entitySet", {
          path: this.codeProperty.path,
          value,
        });
        //this.$store.dispatch("node/setFieldValue", {
        //  path: this.codeProperty.path,
        //  value,
        //});
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
      if (this.editor && this.selectedNode) {
        let existingModel = monaco.editor.getModel(this.selectedNode.id);
        if (!existingModel) {
          const newModel = monaco.editor.createModel(
            this.fieldValue,
            this.codeProperty.language ? this.codeProperty.language : "yaml",
            this.selectedNode.id,
          );
          this.editor.setModel(newModel);
        } else {
          existingModel.setValue(this.fieldValue);
          this.editor.setModel(existingModel);
        }
      }
    },
  },
  async mounted() {
    await this.$store.dispatch("editor/loadEditObject");
    const mountPoint = document.getElementById("monaco-mount");
    if (mountPoint) {
      let editor = monaco.editor.create(mountPoint, {
        theme: "vs-dark",
        automaticLayout: true,
        language: this.codeProperty.language
          ? this.codeProperty.language
          : "yaml",
        scrollBeyondLastLine: false,
        minimap: {
          enabled: false,
        },
      });
      const vx = this;
      editor.onDidBlurEditorText(function () {
        const model = editor.getModel();
        if (model) {
          let value = model.getValue();
          vx.fieldValue = value;
        }
      });
      this.editor = editor;
      this.updateModel();
    } else {
      console.log("failed to mount monaco, element not found");
    }
  },
  watch: {
    selectedNode(currentValue: Node) {
      this.updateModel();
    },
  },
});
</script>
