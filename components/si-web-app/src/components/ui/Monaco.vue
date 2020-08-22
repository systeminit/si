<template>
  <div class="flex flex-col" style="height: 90%">
    <div class="w-full h-full">
      <div id="monaco-mount" class="h-full"></div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState, mapGetters } from "vuex";
import * as monaco from "monaco-editor";

import { Node } from "@/store/modules/node";

interface Data {
  editor: monaco.editor.IEditor | undefined;
}

export default Vue.extend({
  name: "Monaco",
  data(): Data {
    return {
      editor: undefined,
    };
  },
  computed: {
    ...mapGetters({
      codeProperty: "node/codeProperty",
    }),
    ...mapState({
      selectedNode: (state: any): any => state.node.current,
    }),
    fieldValue: {
      set(value: any) {
        console.log("setting the new value", {
          value,
          codePropertyPath: this.codeProperty.path,
        });
        this.$store.dispatch("node/setFieldValue", {
          path: this.codeProperty.path,
          value,
        });
      },
      get(): string {
        if (this.codeProperty) {
          return this.$store.getters["node/getFieldValue"](
            this.codeProperty.path,
          );
        } else {
          return "# No Code!";
        }
      },
    },
  },
  methods: {
    updateModel(): void {
      if (this.editor) {
        let existingModel = monaco.editor.getModel(this.selectedNode.id);
        if (!existingModel) {
          const newModel = monaco.editor.createModel(
            this.fieldValue,
            "yaml",
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
  mounted() {
    const mountPoint = document.getElementById("monaco-mount");
    if (mountPoint) {
      let editor = monaco.editor.create(mountPoint, {
        theme: "vs-dark",
        automaticLayout: true,
        language: "yaml",
        scrollBeyondLastLine: false,
      });
      const vx = this;
      editor.onDidBlurEditorText(function() {
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
